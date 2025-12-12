
package org.globalart.astrum.internal.webhooks;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.capability.CapabilityConfigurationSupport;
import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.ItemselectFormField;
import org.globalart.astrum.formfields.PasswordFormField;
import org.globalart.astrum.formfields.UrlFormField;
import org.globalart.astrum.validation.constraint.Url;
import org.globalart.astrum.validation.constraint.SsrfSafeUrl;
import org.globalart.astrum.webhooks.GlobalWebhook;
import org.globalart.astrum.webhooks.WebhookConfiguration;
import org.globalart.astrum.webhooks.WebhookService;
import org.globalart.astrum.webhooks.WebhookSubscription;

import com.google.common.base.Splitter;
import com.google.common.base.Strings;
import com.google.common.collect.ImmutableSet;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.capability.CapabilityType.capabilityType;

@Component(GlobalWebhookCapability.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class GlobalWebhookCapability
    extends CapabilitySupport<GlobalWebhookCapability.Configuration>
{
  public static final String TYPE_ID = "webhook.global";

  public static final CapabilityType TYPE = capabilityType(TYPE_ID);

  interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Webhook: Global")
    String name();

    @DefaultMessage("Webhook")
    String category();

    @DefaultMessage("Event Types")
    String namesLabel();

    @DefaultMessage("Event types which trigger this Webhook.\n\nNote: The 'firewall_quarantine' event is only emitted for repositories where Repository Firewall is in quarantine or PCCS mode and IQ Server is reachable.")
    String namesHelp();

    @DefaultMessage("URL")
    String urlLabel();

    @DefaultMessage("Send an HTTP POST request to this URL")
    String urlHelp();

    @DefaultMessage("Secret Key")
    String secretLabel();

    @DefaultMessage("Key to use for HMAC payload digest")
    String secretHelp();

    @DefaultMessage("%s")
    String description(String names);
  }

  private static final Messages messages = I18N.create(Messages.class);

  @Autowired
  private WebhookService webhookService;

  private final List<WebhookSubscription> subscriptions = new ArrayList<>();

  @Override
  protected Configuration createConfig(final Map<String, String> properties) throws Exception {
    return new GlobalWebhookCapability.Configuration(properties, secretsService());
  }

  @Override
  protected String renderDescription() {
    return messages.description(String.join(", ", getConfig().names));
  }

  /**
   * Activation is only gated on the standard passivate-during-update condition. Historically an
   * additional {@code capabilityOfTypeActive("firewall.audit")} clause was added whenever the
   * subscribed events included {@code firewall_quarantine}, but ASTRUM-53667 removed that: firewall
   * configuration lives on each repository now (see
   * {@code FirewallCapabilityToRepositoryConfigMigrationStep_2_143}) and the {@code firewall.audit}
   * capability rows no longer exist post-migration, so that clause was permanently false and the
   * capability could never activate. Event emission is already correctly gated per-repository at
   * fire time inside {@code FirewallContributedHandler}; a webhook subscribed to an event that no
   * repository will ever raise is harmless.
   */
  @Override
  public Condition activationCondition() {
    return conditions().capabilities().passivateCapabilityDuringUpdate();
  }

  @Override
  public void onActivate(final Configuration config) {
    webhookService.getWebhooks()
        .stream()
        .filter(webhook -> webhook.getType() == GlobalWebhook.TYPE && config.names.contains(webhook.getName()))
        .forEach(webhook -> subscriptions.add(webhook.subscribe(config)));
  }

  @Override
  public void onPassivate(final Configuration config) {
    subscriptions.forEach(WebhookSubscription::cancel);
    subscriptions.clear();
  }

  public static class Configuration
      extends CapabilityConfigurationSupport
      implements WebhookConfiguration
  {
    private static final String P_NAMES = "names";

    private static final String P_URL = "url";

    private static final String P_SECRET = "secret";

    private static final Splitter LIST_SPLITTER = Splitter.on(',').trimResults().omitEmptyStrings();

    @GlobalWebhookType
    public List<String> names;

    @SsrfSafeUrl
    @Url
    public URI url;

    @Nullable
    private final String secretId;

    private final SecretsService secretsService;

    public Configuration(
        final Map<String, String> properties,
        final SecretsService secretsService)
    {
      this.names = parseList(properties.get(P_NAMES));
      this.url = parseUri(properties.get(P_URL));
      // Store only the secret ID - decrypt on-demand when needed
      this.secretId = Strings.emptyToNull(properties.get(P_SECRET));
      this.secretsService = secretsService;
    }

    private static List<String> parseList(final String value) {
      List<String> result = new ArrayList<>();
      LIST_SPLITTER.split(value).forEach(result::add);
      return result;
    }

    @Override
    public URI getUrl() {
      return url;
    }

    @Nullable
    @Override
    public String getSecret() {
      return decryptSecret(secretId, secretsService);
    }
  }

  @AvailabilityVersion(from = "1.0")
  @Component
  @Qualifier(TYPE_ID)
  public static class Descriptor
      extends CapabilityDescriptorSupport<Configuration>
      implements Taggable
  {
    private final FormField names;

    private final FormField url;

    private final FormField secret;

    public Descriptor() {
      this.names = new ItemselectFormField(
          Configuration.P_NAMES,
          messages.namesLabel(),
          messages.namesHelp(),
          FormField.MANDATORY).withStoreApi("coreui_Webhook.listWithTypeGlobal")
              .withButtons(new String[]{"add", "remove"})
              .withFromTitle("Available")
              .withToTitle("Selected");

      this.url = new UrlFormField(
          Configuration.P_URL,
          messages.urlLabel(),
          messages.urlHelp(),
          FormField.MANDATORY);

      this.secret = new PasswordFormField(
          Configuration.P_SECRET,
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
      return List.of(names, url, secret);
    }

    @Override
    protected Configuration createConfig(final Map<String, String> properties) {
      // This is only used for validation - pass only secretsService as it's required for decryption
      // Use a dummy configuration for validation
      return new Configuration(properties, null);
    }

    @Override
    protected String renderAbout() {
      return render(TYPE_ID + "-about.vm");
    }

    @Override
    public Set<Tag> getTags() {
      return ImmutableSet.of(Tag.categoryTag(messages.category()));
    }
  }
}
