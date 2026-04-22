package org.globalart.astrum.repository.alpine.internal.group;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.concurrent.locks.Lock;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.alpine.AlpineContentFacet;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.internal.AlpineSigningFacet;
import org.globalart.astrum.repository.alpine.internal.apk.ApkIndexArchive;
import org.globalart.astrum.repository.alpine.internal.apk.ApkIndexEntry;
import org.globalart.astrum.repository.cache.RepositoryCacheInvalidationService;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacetImpl;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.alpine.AlpineContentFacet.ATTRIBUTES_KEY;

@Facet.Exposed
@Component
@Qualifier(AlpineFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AlpineGroupFacet
    extends GroupFacetImpl
{
  public static final String MERGE_KEY = "mergeKey";

  public static final String SIGNER = "signer";

  public static final String MEMBERS = "members";

  public static final String PACKAGE_COUNT = "packageCount";

  private final Striped<Lock> indexLocks = Striped.lock(64);

  @Autowired
  public AlpineGroupFacet(
      final RepositoryManager repositoryManager,
      final ConstraintViolationFactory constraintViolationFactory,
      @Qualifier(GroupType.NAME) final Type groupType,
      final RepositoryCacheInvalidationService repositoryCacheInvalidationService)
  {
    super(repositoryManager, constraintViolationFactory, groupType, repositoryCacheInvalidationService);
  }

  public Content mergedIndex(final String path, final Map<Repository, Content> memberIndexes) throws IOException {
    AlpineSigningFacet signing = facet(AlpineSigningFacet.class);
    if (signing.signingKey() == null && memberIndexes.size() == 1) {
      return memberIndexes.values().iterator().next();
    }
    List<MemberIndex> members = new ArrayList<>();
    for (Entry<Repository, Content> entry : memberIndexes.entrySet()) {
      members.add(new MemberIndex(entry.getKey().getName(), entry.getValue()));
    }
    String mergeKey = mergeKey(signing.signerId(), members);
    Optional<Content> cached = cached(path, mergeKey);
    if (cached.isPresent()) {
      return cached.get();
    }
    Lock lock = indexLocks.get(path);
    lock.lock();
    try {
      cached = cached(path, mergeKey);
      if (cached.isPresent()) {
        return cached.get();
      }
      return merge(path, mergeKey, members, signing);
    }
    finally {
      lock.unlock();
    }
  }

  private Content merge(
      final String path,
      final String mergeKey,
      final List<MemberIndex> members,
      final AlpineSigningFacet signing) throws IOException
  {
    Map<String, ApkIndexEntry> entries = new LinkedHashMap<>();
    List<String> merged = new ArrayList<>();
    for (MemberIndex member : members) {
      try (InputStream in = member.openStream()) {
        for (ApkIndexEntry entry : ApkIndexArchive.read(in).entries()) {
          entries.putIfAbsent(entry.key(), entry);
        }
        merged.add(member.repositoryName);
      }
      catch (IOException e) {
        log.warn("Skipping unreadable {} from member {} of group {}: {}", path, member.repositoryName,
            getRepository().getName(), e.getMessage());
      }
    }
    byte[] index = ApkIndexArchive.build(getRepository().getName(), new ArrayList<>(entries.values()),
        signing.signingKey(), Instant.now());
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(MERGE_KEY, mergeKey);
    attributes.put(SIGNER, signing.signerId());
    attributes.put(MEMBERS, merged);
    attributes.put(PACKAGE_COUNT, entries.size());
    log.debug("Merged {} of group {} from {} with {} packages", path, getRepository().getName(), merged,
        entries.size());
    return content().putIndex(path, index, attributes);
  }

  private Optional<Content> cached(final String path, final String mergeKey) {
    Optional<FluentAsset> asset = content().getAsset(path);
    if (asset.isPresent() && mergeKey.equals(asset.get().attributes(ATTRIBUTES_KEY).get(MERGE_KEY, String.class))) {
      return Optional.of(asset.get().download());
    }
    return Optional.empty();
  }

  private static String mergeKey(final String signerId, final List<MemberIndex> members) throws IOException {
    MessageDigest digest = sha256();
    digest.update(signerId.getBytes(StandardCharsets.UTF_8));
    for (MemberIndex member : members) {
      digest.update(("\n" + member.repositoryName + "=" + member.digest()).getBytes(StandardCharsets.UTF_8));
    }
    return HexFormat.of().formatHex(digest.digest());
  }

  private static MessageDigest sha256() {
    try {
      return MessageDigest.getInstance("SHA-256");
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }

  private AlpineContentFacet content() {
    return facet(AlpineContentFacet.class);
  }

  private static final class MemberIndex
  {
    private final String repositoryName;

    private final Content content;

    @Nullable
    private byte[] bytes;

    private MemberIndex(final String repositoryName, final Content content) {
      this.repositoryName = repositoryName;
      this.content = content;
    }

    String digest() throws IOException {
      Asset asset = content.getAttributes().get(Asset.class);
      if (asset != null) {
        Optional<String> checksum = asset.blob().map(AssetBlob::checksums).map(checksums -> checksums.get("sha1"));
        if (checksum.isPresent()) {
          return checksum.get();
        }
      }
      return HexFormat.of().formatHex(sha256().digest(bytes()));
    }

    InputStream openStream() throws IOException {
      return bytes != null ? new ByteArrayInputStream(bytes) : content.openInputStream();
    }

    private byte[] bytes() throws IOException {
      if (bytes == null) {
        try (InputStream in = content.openInputStream()) {
          bytes = in.readAllBytes();
        }
      }
      return bytes;
    }
  }
}
