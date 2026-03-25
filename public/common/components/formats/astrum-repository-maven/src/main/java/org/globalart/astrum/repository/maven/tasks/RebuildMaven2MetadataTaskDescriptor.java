
package org.globalart.astrum.repository.maven.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.CheckboxFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import static org.globalart.astrum.formfields.FormField.OPTIONAL;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link RebuildMaven2MetadataTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class RebuildMaven2MetadataTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.maven.rebuild-metadata";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public static final String GROUPID_FIELD_ID = "groupId";

  public static final String ARTIFACTID_FIELD_ID = "artifactId";

  public static final String BASEVERSION_FIELD_ID = "baseVersion";

  public static final String REBUILD_CHECKSUMS = "rebuildChecksums";

  public static final String CASCADE_REBUILD = "cascadeRebuild";

  public RebuildMaven2MetadataTaskDescriptor() {
    super(TYPE_ID,
        RebuildMaven2MetadataTask.class,
        "Repair - Rebuild Maven repository metadata (maven-metadata.xml)",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the hosted Maven repository to rebuild metadata",
            true).includingAnyOfFormats(Maven2Format.NAME)
                .includingAnyOfTypes(HostedType.NAME)
                .includeAnEntryForAllRepositories(),
        new StringTextFormField(
            GROUPID_FIELD_ID,
            "GroupId",
            "Maven groupId to narrow operation (limit to given groupId only)",
            false),
        new StringTextFormField(
            ARTIFACTID_FIELD_ID,
            "ArtifactId (only if GroupId given)",
            "Maven artifactId to narrow operation (limit to given groupId:artifactId, only used if groupId set)",
            false),
        new StringTextFormField(
            BASEVERSION_FIELD_ID,
            "Base Version (only if ArtifactId given)",
            "Maven base version to narrow operation (limit to given groupId:artifactId:baseVersion, used if groupId " +
                "and artifactId set!)",
            false),
        new CheckboxFormField(
            REBUILD_CHECKSUMS,
            "Rebuild checksums",
            "Compare maven checksum files with recorded metadata, creating files if they are missing and updating " +
                "them if they are incorrect. This can significantly increase the time needed for this task.",
            OPTIONAL).withInitialValue(false),
        new CheckboxFormField(
            CASCADE_REBUILD,
            "Cascade rebuild",
            "If you do not specify groupId and/or artifactId and/or base version, all nested components will be rebuilt. "
                +
                "If there is no groupId - all repository components will be rebuilt; no artifactId - all artifacts related to groupId will be rebuilt etc.",
            OPTIONAL).withInitialValue(true));
  }
}
