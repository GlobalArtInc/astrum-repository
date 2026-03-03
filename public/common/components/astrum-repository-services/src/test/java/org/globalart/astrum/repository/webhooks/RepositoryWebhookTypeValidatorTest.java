
package org.globalart.astrum.repository.webhooks;

import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;

public class RepositoryWebhookTypeValidatorTest
{
  private final RepositoryWebhookTypeValidator validator = new RepositoryWebhookTypeValidator(List.of(
      new TestRepositoryWebhook("webhook1"),
      new TestRepositoryWebhook("webhook2")));

  @Test
  void validatesEmptyWebhookNamesAsValid() {
    List<String> names = List.of();
    boolean result = validator.isValid(names, null);
    assertThat(result, is(true));
  }

  @Test
  void validatesNullWebhookNamesAsValid() {
    boolean result = validator.isValid(null, null);
    assertThat(result, is(true));
  }

  @Test
  void validatesValidWebhookNames() {
    List<String> names = List.of("webhook1", "webhook2");
    boolean result = validator.isValid(names, null);
    assertThat(result, is(true));
  }

  @Test
  void invalidatesInvalidWebhookNames() {
    List<String> names = List.of("webhook1", "invalidWebhook");
    boolean result = validator.isValid(names, null);
    assertThat(result, is(false));
  }

  private static class TestRepositoryWebhook
      extends RepositoryWebhook
  {
    private final String name;

    public TestRepositoryWebhook(String name) {
      this.name = name;
    }

    @Override
    public String getName() {
      return name;
    }
  }
}
