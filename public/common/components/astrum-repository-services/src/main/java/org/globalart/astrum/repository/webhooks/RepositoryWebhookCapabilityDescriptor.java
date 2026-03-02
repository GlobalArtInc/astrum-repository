
package org.globalart.astrum.repository.webhooks;

import java.util.List;
import java.util.Map;
import java.util.Set;
import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.ItemselectFormField;
import org.globalart.astrum.formfields.PasswordFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.formfields.UrlFormField;
import org.globalart.astrum.repository.types.GroupType;

import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapability.TYPE;
import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapability.TYPE_ID;
import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapability.messages;
import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapabilityConfiguration.P_NAMES;
import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapabilityConfiguration.P_REPOSITORY;
import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapabilityConfiguration.P_SECRET;
import static org.globalart.astrum.repository.webhooks.RepositoryWebhookCapabilityConfiguration.P_URL;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(TYPE_ID)
public class RepositoryWebhookCapabilityDescriptor
    extends CapabilityDescriptorSupport<RepositoryWebhookCapabilityConfiguration>
    implements Taggable
{
  private final FormField<String> repository;

  private final FormField<String> names;

  private final FormField<String> url;

  private final FormField<String> secret;

  public RepositoryWebhookCapabilityDescriptor() {
    setExposed(true);
    setHidden(false);

    this.repository = new RepositoryCombobox(
        P_REPOSITORY,
        messages.repositoryLabel(),
        messages.repositoryHelp(),
        FormField.MANDATORY).excludingAnyOfTypes(GroupType.NAME);

    this.names = new ItemselectFormField(
        P_NAMES,
        messages.namesLabel(),
        messages.namesHelp(),
        FormField.MANDATORY).withStoreApi("coreui_Webhook.listWithTypeRepository")
            .withButtons("add", "remove")
            .withFromTitle("Available")
            .withToTitle("Selected");

    this.url = new UrlFormField(
        P_URL,
        messages.urlLabel(),
        messages.urlHelp(),
        FormField.MANDATORY);

    this.secret = new PasswordFormField(
        P_SECRET,
        messages.secretLabel(),
        messages.secretHelp(),
        FormField.OPTIONAL);
  }

  @Override
  public CapabilityType type() {
    return TYPE;
  }

  @Override
  public String name() {
    return messages.name();
  }

  @Override
  public List<FormField> formFields() {
    return List.of(repository, names, url, secret);
  }

  @Override
  protected RepositoryWebhookCapabilityConfiguration createConfig(final Map<String, String> properties) {
    // This is only used for validation - pass null for secrets as they won't be accessed during validation
    return new RepositoryWebhookCapabilityConfiguration(properties, null);
  }

  @Override
  protected String renderAbout() {
    return render(TYPE_ID + "-about.vm");
  }

  @Override
  public Set<Tag> getTags() {
    return Set.of(Tag.categoryTag(messages.category()));
  }

  @Override
  protected Set<String> uniqueProperties() {
    return Set.of(P_REPOSITORY, P_URL);
  }
}
