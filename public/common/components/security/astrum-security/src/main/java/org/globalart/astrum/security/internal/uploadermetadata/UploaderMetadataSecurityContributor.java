
package org.globalart.astrum.security.internal.uploadermetadata;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import org.springframework.stereotype.Component;

@Component
public class UploaderMetadataSecurityContributor
    extends SecurityContributorSupport
{
  public static final String UPLOADER_METADATA_DOMAIN = "uploader-metadata";

  public static final String UPLOADER_METADATA_READ_PRIV_ID = "as-uploader-metadata-read";

  public static final String UPLOADER_METADATA_READ_PERMISSION = "astrum:" + UPLOADER_METADATA_DOMAIN + ":read";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();
    config.addPrivilege(
        createApplicationPrivilege(
            UPLOADER_METADATA_READ_PRIV_ID,
            READ_DESCRIPTION_BASE + "Uploader Metadata",
            UPLOADER_METADATA_DOMAIN,
            ACTION_READ));
    return config;
  }
}
