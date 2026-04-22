package org.globalart.astrum.repository.alpine.internal;

import java.nio.charset.StandardCharsets;
import java.util.HashSet;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nullable;

import jakarta.validation.ConstraintViolation;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.internal.apk.ApkSigningKey;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.validation.ConstraintViolations.maybePropagate;

@Facet.Exposed
@Component
@Qualifier(AlpineFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AlpineSigningFacet
    extends FacetSupport
{
  public static final String CONFIG_KEY = "alpineSigning";

  public static final String KEYPAIR = "keypair";

  public static final String PASSPHRASE = "passphrase";

  public static final String UNSIGNED = "unsigned";

  public static final String PUBLIC_KEY_CONTENT_TYPE = "application/x-pem-file";

  private final ConstraintViolationFactory constraintViolationFactory;

  @Nullable
  private volatile ApkSigningKey signingKey;

  @Autowired
  public AlpineSigningFacet(final ConstraintViolationFactory constraintViolationFactory) {
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

  @Nullable
  public ApkSigningKey signingKey() {
    return signingKey;
  }

  public String signerId() {
    ApkSigningKey key = signingKey;
    return key == null ? UNSIGNED : key.id();
  }

  public Optional<Content> publicKey(final String fileName) {
    ApkSigningKey key = signingKey;
    if (key == null || !key.keyFileName().equals(fileName)) {
      return Optional.empty();
    }
    return Optional.of(new Content(
        new BytesPayload(key.publicKeyPem().getBytes(StandardCharsets.US_ASCII), PUBLIC_KEY_CONTENT_TYPE)));
  }

  @Nullable
  private static ApkSigningKey readKey(final Configuration configuration) {
    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    Map<String, Object> signing = attributes == null ? null : attributes.get(CONFIG_KEY);
    if (signing == null || !(signing.get(KEYPAIR) instanceof String keypair) || keypair.isBlank()) {
      return null;
    }
    Object passphrase = signing.get(PASSPHRASE);
    return ApkSigningKey.parse(configuration.getRepositoryName(), keypair,
        passphrase instanceof String value ? value : null);
  }
}
