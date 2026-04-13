
package org.globalart.astrum.repository.apt.internal.gpg;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.apt.internal.AptMimeTypes;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.security.GpgUtils;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.google.common.annotations.VisibleForTesting;
import org.apache.commons.io.IOUtils;
import org.bouncycastle.bcpg.ArmoredOutputStream;
import org.bouncycastle.bcpg.BCPGOutputStream;
import org.bouncycastle.openpgp.PGPPublicKey;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Signs an Apt metadata by using PGP signing key pair.
 *
 * @since 3.17
 */
@Component
@Facet.Exposed
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AptSigningFacet
    extends FacetSupport
{
  public static final String CONFIG_KEY = "aptSigning";

  @VisibleForTesting
  static class Config
  {
    @NotNull(groups = {HostedType.ValidationGroup.class, GroupType.ValidationGroup.class})
    public String keypair;

    public String passphrase = "";
  }

  private Config config;

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    facet(ConfigurationFacet.class).validateSection(
        configuration,
        CONFIG_KEY,
        Config.class,
        Default.class,
        getRepository().getType().getValidationGroup());
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    config = facet(ConfigurationFacet.class).readSection(configuration, CONFIG_KEY, Config.class);
  }

  @Override
  protected void doDestroy() throws Exception {
    config = null;
  }

  /**
   * Checks if signing is configured for this repository.
   *
   * @return true if a keypair is configured
   */
  public boolean isConfigured() {
    return config != null && config.keypair != null && !config.keypair.isBlank();
  }

  public Content getPublicKey() throws IOException {
    ByteArrayOutputStream buffer = new ByteArrayOutputStream();
    PGPPublicKey publicKey = GpgUtils.getPublicKey(config.keypair);
    try (BCPGOutputStream os = new BCPGOutputStream(new ArmoredOutputStream(buffer))) {
      publicKey.encode(os);
    }

    return new Content(new BytesPayload(buffer.toByteArray(), AptMimeTypes.PUBLICKEY));
  }

  public byte[] signInline(final String input) throws IOException {
    return GpgUtils.signInline(input, config.keypair, config.passphrase);
  }

  public byte[] signExternal(final String input) throws IOException {
    try (InputStream is = IOUtils.toInputStream(input, StandardCharsets.UTF_8)) {
      return GpgUtils.signExternal(is, config.keypair, config.passphrase);
    }
  }
}
