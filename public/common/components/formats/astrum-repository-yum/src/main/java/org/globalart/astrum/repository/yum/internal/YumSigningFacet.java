package org.globalart.astrum.repository.yum.internal;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.HashSet;
import java.util.HexFormat;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nullable;

import jakarta.validation.ConstraintViolation;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.security.GpgUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import org.bouncycastle.bcpg.ArmoredOutputStream;
import org.bouncycastle.bcpg.BCPGOutputStream;
import org.bouncycastle.openpgp.PGPPublicKey;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.validation.ConstraintViolations.maybePropagate;

@Facet.Exposed
@Component
@Qualifier(YumFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class YumSigningFacet
    extends FacetSupport
{
  public static final String CONFIG_KEY = "yumSigning";

  public static final String KEYPAIR = "keypair";

  public static final String PASSPHRASE = "passphrase";

  public static final String UNSIGNED = "unsigned";

  public static final String SIGNATURE_CONTENT_TYPE = "application/pgp-signature";

  public static final String KEY_CONTENT_TYPE = "application/pgp-keys";

  private final ConstraintViolationFactory constraintViolationFactory;

  @Nullable
  private volatile SigningKey signingKey;

  @Autowired
  public YumSigningFacet(final ConstraintViolationFactory constraintViolationFactory) {
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
  }

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    Set<ConstraintViolation<?>> violations = new HashSet<>();
    try {
      readKey(configuration);
    }
    catch (IllegalArgumentException e) {
      violations.add(constraintViolationFactory.createViolation(CONFIG_KEY + "." + KEYPAIR, e.getMessage()));
    }
    maybePropagate(violations, log);
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    signingKey = readKey(configuration);
  }

  @Override
  protected void doDestroy() throws Exception {
    signingKey = null;
  }

  public boolean isConfigured() {
    return signingKey != null;
  }

  public String signerId() {
    SigningKey key = signingKey;
    return key == null ? UNSIGNED : key.fingerprint();
  }

  public Optional<byte[]> sign(final byte[] data) throws IOException {
    SigningKey key = signingKey;
    if (key == null) {
      return Optional.empty();
    }
    return Optional.of(GpgUtils.signExternal(new ByteArrayInputStream(data), key.keypair(), key.passphrase()));
  }

  public Optional<Content> publicKey() {
    SigningKey key = signingKey;
    if (key == null) {
      return Optional.empty();
    }
    return Optional.of(new Content(new BytesPayload(key.armoredPublicKey(), KEY_CONTENT_TYPE)));
  }

  @Nullable
  private static SigningKey readKey(final Configuration configuration) {
    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    Map<String, Object> signing = attributes == null ? null : attributes.get(CONFIG_KEY);
    if (signing == null || !(signing.get(KEYPAIR) instanceof String keypair) || keypair.isBlank()) {
      return null;
    }
    String passphrase = signing.get(PASSPHRASE) instanceof String value ? value : null;
    try {
      if (!GpgUtils.isConfigured(keypair, passphrase)) {
        throw new IllegalArgumentException("The signing key cannot be read with the given passphrase");
      }
      PGPPublicKey publicKey = GpgUtils.getPublicKey(keypair);
      ByteArrayOutputStream armored = new ByteArrayOutputStream();
      try (BCPGOutputStream out = new BCPGOutputStream(new ArmoredOutputStream(armored))) {
        publicKey.encode(out);
      }
      return new SigningKey(keypair, passphrase, HexFormat.of().formatHex(publicKey.getFingerprint()),
          armored.toByteArray());
    }
    catch (IOException | RuntimeException e) {
      if (e instanceof IllegalArgumentException) {
        throw (IllegalArgumentException) e;
      }
      throw new IllegalArgumentException("The signing key is not an armored PGP private key: " + e.getMessage());
    }
  }

  private record SigningKey(String keypair, @Nullable String passphrase, String fingerprint, byte[] armoredPublicKey)
  {
  }
}
