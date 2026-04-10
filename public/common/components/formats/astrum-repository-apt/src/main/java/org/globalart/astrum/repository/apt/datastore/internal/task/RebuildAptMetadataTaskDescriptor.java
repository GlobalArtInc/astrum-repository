
package org.globalart.astrum.repository.apt.datastore.internal.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.CheckboxFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import static org.globalart.astrum.formfields.FormField.OPTIONAL;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class RebuildAptMetadataTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.apt.rebuild.metadata";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public static final String APT_METADATA_FULL_REBUILD = "rebuildAptMetadataFullRebuild";

  public static final String APT_PROXY_RESET_METADATA = "resetProxyMetadata";

  public RebuildAptMetadataTaskDescriptor() {
    super(TYPE_ID,
        RebuildAptMetadataTask.class,
        "Apt - Rebuild Apt metadata",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the Apt repository to rebuild metadata",
            true).includingAnyOfFormats(AptFormat.NAME)
                .includingAnyOfTypes(HostedType.NAME, ProxyType.NAME)
                .includeAnEntryForAllRepositories()
                .withListener("select", "updateAptRebuildCheckboxVisibility")
                .withListener("afterrender", "updateAptRebuildCheckboxVisibility"),
        new CheckboxFormField(
            APT_METADATA_FULL_REBUILD,
            "Full rebuild (hosted only)",
            "Repopulate apt_key_value table and execute full rebuild of metadata. Only applies to hosted repositories.",
            OPTIONAL),
        new CheckboxFormField(
            APT_PROXY_RESET_METADATA,
            "Reset proxy metadata",
            "Clear all generated metadata before rebuild. Use this after changing the upstream URL. Only applies to proxy repositories.",
            OPTIONAL));
  }
}
