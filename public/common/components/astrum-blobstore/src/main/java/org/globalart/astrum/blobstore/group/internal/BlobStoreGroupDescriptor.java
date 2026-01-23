
package org.globalart.astrum.blobstore.group.internal;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Predicate;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.blobstore.BlobStoreDescriptor;
import org.globalart.astrum.blobstore.BlobStoreDescriptorSupport;
import org.globalart.astrum.blobstore.BlobStoreUtil;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;
import org.globalart.astrum.blobstore.group.BlobStoreGroupService;
import org.globalart.astrum.blobstore.group.FillPolicy;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.ItemselectFormField;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.apache.commons.lang3.StringUtils;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Streams.stream;
import static java.lang.String.format;
import static java.util.Arrays.asList;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toMap;
import static org.globalart.astrum.blobstore.group.BlobStoreGroup.CONFIG_KEY;
import static org.globalart.astrum.blobstore.group.BlobStoreGroup.FILL_POLICY_KEY;
import static org.globalart.astrum.blobstore.group.BlobStoreGroup.MEMBERS_KEY;
import static org.globalart.astrum.blobstore.group.BlobStoreGroupConfigurationHelper.memberNames;
import static org.globalart.astrum.formfields.FormField.MANDATORY;

/**
 * A {@link BlobStoreDescriptor} for {@link BlobStoreGroup}.
 *
 * @since 3.14
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(BlobStoreGroup.TYPE)
public class BlobStoreGroupDescriptor
    extends BlobStoreDescriptorSupport
{
  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Group")
    String name();

    @DefaultMessage("Members")
    String membersLabel();

    @DefaultMessage("Fill Policy")
    String fillPolicyLabel();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final BlobStoreManager blobStoreManager;

  private final BlobStoreUtil blobStoreUtil;

  private final Provider<BlobStoreGroupService> blobStoreGroupService;

  private final ItemselectFormField members;

  private final FormField fillPolicy;

  private final Map<String, FillPolicy> fillPolicies;

  @Autowired
  public BlobStoreGroupDescriptor(
      @Lazy final BlobStoreManager blobStoreManager,
      final BlobStoreUtil blobStoreUtil,
      final ObjectProvider<BlobStoreGroupService> blobStoreGroupService,
      final BlobStoreQuotaService quotaService,
      final List<FillPolicy> fillPoliciesList)
  {
    super(quotaService);
    this.blobStoreManager = checkNotNull(blobStoreManager);
    this.blobStoreUtil = checkNotNull(blobStoreUtil);
    this.blobStoreGroupService = checkNotNull(blobStoreGroupService)::getIfAvailable;
    this.fillPolicies = QualifierUtil.buildQualifierBeanMap(checkNotNull(fillPoliciesList));
    this.members = new ItemselectFormField(
        MEMBERS_KEY,
        messages.membersLabel(),
        null,
        MANDATORY);
    this.members.setStoreApi("coreui_Blobstore.readGroupable");
    this.members.setIdMapping("name");
    this.members.setButtons("up", "add", "remove", "down");
    this.members.setFromTitle("Available Blob Stores");
    this.members.setToTitle("Selected Blob Stores");
    this.fillPolicy = new ComboboxFormField<String>(
        BlobStoreGroup.FILL_POLICY_KEY,
        messages.fillPolicyLabel(),
        null,
        FormField.MANDATORY).withStoreApi("coreui_Blobstore.fillPolicies");
    this.fillPolicy.getAttributes()
        .put("options", fillPolicies.entrySet()
            .stream()
            .collect(
                toMap(Map.Entry::getKey, e -> e.getValue().getName())));
  }

  @Override
  public String getId() {
    return "group";
  }

  @Override
  public String getName() {
    return messages.name();
  }

  @Override
  public List<FormField> getFormFields() {
    List<String> blobStores = stream(blobStoreManager.browse())
        .map(BlobStore::getBlobStoreConfiguration)
        .map(BlobStoreConfiguration::getName)
        .collect(toList());
    this.members.getAttributes().put("options", blobStores);

    return asList(members, fillPolicy);
  }

  @Override
  public boolean isModifiable() {
    return isEnabled();
  }

  @Override
  public void validateConfig(final BlobStoreConfiguration config) {
    super.validateConfig(config);
    validateEnabled();
    String name = config.getName();

    String fillPolicy = config.attributes(CONFIG_KEY).get(FILL_POLICY_KEY, String.class);
    if (StringUtils.isBlank(fillPolicy)) {
      throw new ValidationErrorsException("Blob store group requires a fill policy configuration");
    }
    if (!fillPolicies.containsKey(fillPolicy)) {
      throw new ValidationErrorsException("Blob store group requires a valid fill policy name, options include [{}]",
          String.join(", ", fillPolicies.keySet()));
    }

    List<String> memberNames = config.attributes(CONFIG_KEY).get(MEMBERS_KEY, List.class);
    validateNotEmptyOrSelfReferencing(name, memberNames);
    validateEligibleMembers(name, memberNames);
    validateOnlyEmptyOrNotWritableExistingMembersRemoved(name, memberNames);
  }

  private void validateEnabled() {
    if (!isEnabled()) {
      throw new ValidationErrorsException("Blob store groups are not enabled");
    }
  }

  private void validateNotEmptyOrSelfReferencing(final String name, final List<String> memberNames) {
    if (memberNames == null || memberNames.isEmpty()) {
      throw new ValidationErrorsException("Blob Store '" + name + "' cannot be empty");
    }

    if (memberNames.contains(name)) {
      throw new ValidationErrorsException("Blob Store '" + name + "' cannot contain itself");
    }
  }

  private void validateEligibleMembers(final String name, final List<String> memberNames) {
    for (String memberName : memberNames) {
      BlobStore member = blobStoreManager.get(memberName);
      if (!member.isGroupable()) {
        BlobStoreConfiguration memberConfig = member.getBlobStoreConfiguration();
        throw new ValidationErrorsException(
            format("Blob Store '%s' is of type '%s' and is not eligible to be a group member", memberName,
                memberConfig.getType()));
      }

      if (blobStoreManager.hasConflictingTasks(memberName)) {
        throw new ValidationErrorsException(
            format("Blob Store '%s' has conflicting tasks running and is not eligible to be a group member",
                memberName));
      }

      // target member may not be a member of a different group
      Predicate<String> sameGroup = name::equals;
      blobStoreManager.getParent(memberName).filter(sameGroup.negate()).ifPresent(groupName -> {
        throw new ValidationErrorsException(
            format("Blob Store '%s' is already a member of Blob Store Group '%s'", memberName, groupName));
      });

      // target member may not be set as repository storage
      int repoCount = blobStoreUtil.usageCount(memberName);
      if (repoCount > 0) {
        throw new ValidationErrorsException(format(
            "Blob Store '%s' is set as storage for %s repositories and is not eligible to be a group member",
            memberName, repoCount));
      }
    }
  }

  private void validateOnlyEmptyOrNotWritableExistingMembersRemoved(final String name, final List<String> memberNames) {
    BlobStore blobStore = blobStoreManager.get(name);
    if (blobStore != null) {
      BlobStoreConfiguration currentConfiguration = blobStore.getBlobStoreConfiguration();
      if (currentConfiguration != null && currentConfiguration.getType().equals(BlobStoreGroup.TYPE)) {
        for (String existingMemberName : memberNames(currentConfiguration)) {
          if (!memberNames.contains(existingMemberName)) {
            BlobStore existingMember = blobStoreManager.get(existingMemberName);
            if (existingMember.isWritable() || !existingMember.isEmpty()) {
              throw new ValidationErrorsException(
                  format("Blob Store '%s' cannot be removed from Blob Store Group '%s', " +
                      "use 'Admin - Remove a member from a blob store group' task instead",
                      existingMemberName, name));
            }
          }
        }
      }
    }
  }

  @Override
  public boolean isEnabled() {
    return Optional.of(blobStoreGroupService)
        .map(Provider::get)
        .map(BlobStoreGroupService::isEnabled)
        .orElse(false);
  }

  @Override
  public boolean configHasDependencyOn(final BlobStoreConfiguration config, final String blobStoreName) {
    return memberNames(config).contains(blobStoreName);
  }
}
