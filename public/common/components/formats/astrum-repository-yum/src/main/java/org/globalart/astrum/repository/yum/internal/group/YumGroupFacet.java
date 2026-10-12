package org.globalart.astrum.repository.yum.internal.group;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.locks.Lock;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.RepositoryCacheInvalidationService;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacetImpl;
import org.globalart.astrum.repository.group.GroupHandler.DispatchedRepositories;
import org.globalart.astrum.repository.http.HttpConditions;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.ContentTypes;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Headers;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumContentFacet;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.YumPaths;
import org.globalart.astrum.repository.yum.internal.YumSigningFacet;
import org.globalart.astrum.repository.yum.internal.repodata.Decompression;
import org.globalart.astrum.repository.yum.internal.repodata.GeneratedMetadata;
import org.globalart.astrum.repository.yum.internal.repodata.MemberRepodata;
import org.globalart.astrum.repository.yum.internal.repodata.Repodata;
import org.globalart.astrum.repository.yum.internal.repodata.RepodataMerger;
import org.globalart.astrum.repository.yum.internal.repodata.Repomd;
import org.globalart.astrum.repository.yum.internal.repodata.RepomdData;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.net.HttpHeaders;
import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.YumContentFacet.ATTRIBUTES_KEY;

@Facet.Exposed
@Component
@Qualifier(YumFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class YumGroupFacet
    extends GroupFacetImpl
{
  public static final String MERGE_KEY = "mergeKey";

  public static final String MEMBERS = "members";

  public static final String FILES = "files";

  public static final String PACKAGE_COUNT = "packageCount";

  private final Striped<Lock> locks = Striped.lock(64);

  @Autowired
  public YumGroupFacet(
      final RepositoryManager repositoryManager,
      final ConstraintViolationFactory constraintViolationFactory,
      @Qualifier(GroupType.NAME) final Type groupType,
      final RepositoryCacheInvalidationService repositoryCacheInvalidationService)
  {
    super(repositoryManager, constraintViolationFactory, groupType, repositoryCacheInvalidationService);
  }

  public record GroupRepomd(Content content, @Nullable Repository passedThrough)
  {
  }

  public Optional<GroupRepomd> repomd(final Context context, final String base) throws Exception {
    Map<Repository, Payload> responses = new LinkedHashMap<>();
    DispatchedRepositories dispatched = new DispatchedRepositories();
    dispatched.add(getRepository());
    for (Repository member : members()) {
      if (member.getConfiguration() != null && !member.getConfiguration().isOnline()) {
        continue;
      }
      Response response = dispatch(member, context, YumPaths.repomdPath(base), dispatched);
      if (response.getStatus().getCode() == HttpStatus.OK && response.getPayload() != null) {
        responses.put(member, response.getPayload());
      }
    }
    if (responses.isEmpty()) {
      return Optional.empty();
    }
    if (responses.size() == 1) {
      Entry<Repository, Payload> only = responses.entrySet().iterator().next();
      Payload payload = only.getValue();
      return Optional.of(new GroupRepomd(payload instanceof Content content ? content : new Content(payload),
          only.getKey()));
    }
    Map<Repository, byte[]> repomds = new LinkedHashMap<>();
    for (Entry<Repository, Payload> entry : responses.entrySet()) {
      try {
        repomds.put(entry.getKey(), read(entry.getValue()));
      }
      catch (IOException e) {
        log.warn("Skipping unreadable {} of member {} of group {}: {}", YumPaths.repomdPath(base),
            entry.getKey().getName(), getRepository().getName(), e.getMessage());
      }
    }
    if (repomds.isEmpty()) {
      return Optional.empty();
    }
    YumSigningFacet signing = facet(YumSigningFacet.class);
    String mergeKey = mergeKey(signing.signerId(), repomds);
    Optional<Content> cached = cached(base, mergeKey);
    if (cached.isPresent()) {
      return Optional.of(new GroupRepomd(cached.get(), null));
    }
    Lock lock = locks.get(base);
    lock.lock();
    try {
      cached = cached(base, mergeKey);
      if (cached.isPresent()) {
        return Optional.of(new GroupRepomd(cached.get(), null));
      }
      return Optional.of(new GroupRepomd(merge(context, base, mergeKey, repomds), null));
    }
    finally {
      lock.unlock();
    }
  }

  public Optional<Content> getMetadata(final String path) {
    return content().get(path);
  }

  private Content merge(
      final Context context,
      final String base,
      final String mergeKey,
      final Map<Repository, byte[]> repomds) throws IOException
  {
    List<MemberRepodata> members = new ArrayList<>();
    for (Entry<Repository, byte[]> entry : repomds.entrySet()) {
      Repomd repomd;
      try (InputStream in = new ByteArrayInputStream(entry.getValue())) {
        repomd = Repomd.parse(in);
      }
      catch (IOException e) {
        log.warn("Skipping unreadable {} of member {} of group {}: {}", YumPaths.repomdPath(base),
            entry.getKey().getName(), getRepository().getName(), e.getMessage());
        continue;
      }
      members.add(new DispatchedMember(entry.getKey(), repomd, context, base));
    }
    YumContentFacet content = content();
    try (Repodata repodata = RepodataMerger.merge(members, Instant.now().getEpochSecond())) {
      List<String> files = new ArrayList<>();
      for (GeneratedMetadata file : repodata.files()) {
        content.putMetadata(YumPaths.repodataDirectory(base) + "/" + file.fileName(), AssetKind.REPODATA,
            file.file(), ContentTypes.APPLICATION_GZIP, Map.of());
        files.add(file.fileName());
      }
      byte[] repomd = repodata.repomd().toXml();
      Set<String> keep = new HashSet<>(files);
      content.getAsset(YumPaths.repomdPath(base)).ifPresent(previous -> keep.addAll(filesOf(previous)));
      Map<String, Object> attributes = new LinkedHashMap<>();
      attributes.put(MERGE_KEY, mergeKey);
      attributes.put(MEMBERS, members.stream().map(MemberRepodata::name).toList());
      attributes.put(FILES, files);
      attributes.put(PACKAGE_COUNT, repodata.packages());
      Content stored = content.putMetadata(YumPaths.repomdPath(base), AssetKind.REPOMD, repomd,
          ContentTypes.APPLICATION_XML, attributes);
      String directory = YumPaths.repodataDirectory(base);
      for (FluentAsset asset : content.assetsUnder(directory, AssetKind.REPODATA)) {
        String name = YumPaths.fileName(asset.path());
        if (YumPaths.directory(asset.path()).equals(directory) && !keep.contains(name)) {
          asset.delete();
        }
      }
      log.debug("Merged {} of group {} from {} with {} packages", YumPaths.repomdPath(base),
          getRepository().getName(), attributes.get(MEMBERS), repodata.packages());
      return stored;
    }
  }

  private Optional<Content> cached(final String base, final String mergeKey) {
    Optional<FluentAsset> asset = content().getAsset(YumPaths.repomdPath(base));
    if (asset.isPresent() && mergeKey.equals(asset.get().attributes(ATTRIBUTES_KEY).get(MERGE_KEY, String.class))) {
      return Optional.of(asset.get().download());
    }
    return Optional.empty();
  }

  @SuppressWarnings("unchecked")
  private static List<String> filesOf(final FluentAsset repomd) {
    Object files = repomd.attributes(ATTRIBUTES_KEY).get(FILES);
    return files instanceof List<?> list ? (List<String>) list : List.of();
  }

  static String mergeKey(final String signerId, final Map<Repository, byte[]> repomds) {
    MessageDigest digest = sha256();
    digest.update(signerId.getBytes(StandardCharsets.UTF_8));
    for (Entry<Repository, byte[]> entry : repomds.entrySet()) {
      digest.update(("\n" + entry.getKey().getName() + "=" + HexFormat.of().formatHex(sha256().digest(entry.getValue())))
          .getBytes(StandardCharsets.UTF_8));
    }
    return HexFormat.of().formatHex(digest.digest());
  }

  public static Response dispatch(
      final Repository member,
      final Context context,
      final String path,
      final DispatchedRepositories dispatched) throws Exception
  {
    Request original = context.getRequest();
    Map<String, Object> attributes = new HashMap<>(original.getAttributes().backing());
    attributes.remove(DispatchedRepositories.class.getName());
    attributes.remove(HttpConditions.HTTP_CONDITIONS);
    AttributesMap requestAttributes = new AttributesMap(attributes);
    requestAttributes.set(DispatchedRepositories.class, dispatched);
    Headers headers = new Headers();
    for (String name : original.getHeaders().names()) {
      if (!CONDITIONAL_HEADERS.contains(name.toLowerCase())) {
        headers.set(name, original.getHeaders().getAll(name));
      }
    }
    Request request = new Request.Builder()
        .copy(original)
        .attributes(requestAttributes)
        .headers(headers)
        .parameters(new Parameters())
        .action(HttpMethods.GET)
        .path(path)
        .build();
    return member.facet(ViewFacet.class).dispatch(request, context);
  }

  private static final Set<String> CONDITIONAL_HEADERS = Set.of(
      HttpHeaders.IF_MODIFIED_SINCE.toLowerCase(),
      HttpHeaders.IF_UNMODIFIED_SINCE.toLowerCase(),
      HttpHeaders.IF_MATCH.toLowerCase(),
      HttpHeaders.IF_NONE_MATCH.toLowerCase(),
      HttpHeaders.RANGE.toLowerCase(),
      HttpHeaders.IF_RANGE.toLowerCase());

  private static byte[] read(final Payload payload) throws IOException {
    try (InputStream in = payload.openInputStream()) {
      return in.readAllBytes();
    }
    finally {
      payload.close();
    }
  }

  private static MessageDigest sha256() {
    try {
      return MessageDigest.getInstance("SHA-256");
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }

  private YumContentFacet content() {
    return facet(YumContentFacet.class);
  }

  private static final class DispatchedMember
      implements MemberRepodata
  {
    private final Repository member;

    private final Repomd repomd;

    private final Context context;

    private final String base;

    private DispatchedMember(final Repository member, final Repomd repomd, final Context context, final String base) {
      this.member = member;
      this.repomd = repomd;
      this.context = context;
      this.base = base;
    }

    @Override
    public String name() {
      return member.getName();
    }

    @Override
    public Repomd repomd() {
      return repomd;
    }

    @Override
    public InputStream open(final RepomdData data) throws IOException {
      String path;
      try {
        path = YumPaths.normalize(base + "/" + data.location());
      }
      catch (IllegalArgumentException e) {
        throw new IOException(e.getMessage());
      }
      Response response;
      try {
        DispatchedRepositories dispatched = new DispatchedRepositories();
        response = dispatch(member, context, path, dispatched);
      }
      catch (IOException e) {
        throw e;
      }
      catch (Exception e) {
        throw new IOException("Unable to fetch " + path + " from " + member.getName() + ": " + e.getMessage(), e);
      }
      if (response.getStatus().getCode() != HttpStatus.OK || response.getPayload() == null) {
        throw new IOException("Member " + member.getName() + " answered " + response.getStatus().getCode()
            + " for " + path);
      }
      return Decompression.open(response.getPayload().openInputStream());
    }
  }
}
