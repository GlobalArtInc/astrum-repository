package org.globalart.astrum.repository.terraform.internal.signing;

import java.util.HashSet;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nullable;
import jakarta.validation.ConstraintViolation;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.validation.ConstraintViolations.maybePropagate;

/**
 * The OpenPGP key of a hosted repository ({@code attributes.terraformSigning}). With a key, the repository signs
 * the checksums of provider versions uploaded without a signature of their own, so Terraform can install them
 * through the provider registry protocol.
 */
@Facet.Exposed
@Component
@Qualifier(TerraformFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TerraformSigningFacet
    extends FacetSupport
{
  public static final String CONFIG_KEY = "terraformSigning";

  public static final String KEYPAIR = "keypair";

  public static final String PASSPHRASE = "passphrase";

  private final ConstraintViolationFactory constraintViolationFactory;

  @Nullable
  private volatile Pgp.SigningKey signingKey;

  @Autowired
  public TerraformSigningFacet(final ConstraintViolationFactory constraintViolationFactory) {
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

  public Optional<Pgp.SigningKey> signingKey() {
    return Optional.ofNullable(signingKey);
  }

  @Nullable
  static Pgp.SigningKey readKey(final Configuration configuration) {
    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    Map<String, Object> signing = attributes == null ? null : attributes.get(CONFIG_KEY);
    if (signing == null || !(signing.get(KEYPAIR) instanceof String keypair) || keypair.isBlank()) {
      return null;
    }
    Object passphrase = signing.get(PASSPHRASE);
    return Pgp.readSigningKey(keypair, passphrase instanceof String value ? value : null);
  }
}
