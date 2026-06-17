
package org.globalart.astrum.coreui;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import org.globalart.astrum.repository.webhooks.RepositoryWebhook;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.webhooks.GlobalWebhook;
import org.globalart.astrum.webhooks.Webhook;
import org.globalart.astrum.webhooks.WebhookService;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class WebhookComponentTest
{
  @Mock
  WebhookService webhookService;

  @InjectMocks
  WebhookComponent underTest;

  @Test
  void listWithTypeGlobalReturnsGlobalWebhooks() {
    Webhook globalWebhook = createGlobalWebhook("globalHook");
    Webhook repositoryWebhook = createRepositoryWebhook("repoHook");

    when(webhookService.getWebhooks()).thenReturn(Arrays.asList(globalWebhook, repositoryWebhook));

    List<ReferenceXO> result = underTest.listWithTypeGlobal();

    assertThat(result, hasSize(1));
    assertThat(result.get(0).getId(), is("globalHook"));
    assertThat(result.get(0).getName(), is("globalHook"));
  }

  @Test
  void listWithTypeRepositoryReturnsRepositoryWebhooks() {
    Webhook globalWebhook = createGlobalWebhook("globalHook");
    Webhook repositoryWebhook = createRepositoryWebhook("repoHook");

    when(webhookService.getWebhooks()).thenReturn(Arrays.asList(globalWebhook, repositoryWebhook));

    List<ReferenceXO> result = underTest.listWithTypeRepository();

    assertThat(result, hasSize(1));
    assertThat(result.get(0).getId(), is("repoHook"));
    assertThat(result.get(0).getName(), is("repoHook"));
  }

  @Test
  void listWithTypeGlobalReturnsEmptyWhenNoGlobalWebhooks() {
    Webhook repositoryWebhook = createRepositoryWebhook("repoHook");

    when(webhookService.getWebhooks()).thenReturn(Collections.singletonList(repositoryWebhook));

    List<ReferenceXO> result = underTest.listWithTypeGlobal();

    assertThat(result, is(empty()));
  }

  @Test
  void listWithTypeRepositoryReturnsEmptyWhenNoRepositoryWebhooks() {
    Webhook globalWebhook = createGlobalWebhook("globalHook");

    when(webhookService.getWebhooks()).thenReturn(Collections.singletonList(globalWebhook));

    List<ReferenceXO> result = underTest.listWithTypeRepository();

    assertThat(result, is(empty()));
  }

  @Test
  void listWithTypeGlobalReturnsEmptyWhenNoWebhooks() {
    when(webhookService.getWebhooks()).thenReturn(Collections.emptyList());

    List<ReferenceXO> result = underTest.listWithTypeGlobal();

    assertThat(result, is(empty()));
  }

  @Test
  void listWithTypeRepositoryReturnsEmptyWhenNoWebhooks() {
    when(webhookService.getWebhooks()).thenReturn(Collections.emptyList());

    List<ReferenceXO> result = underTest.listWithTypeRepository();

    assertThat(result, is(empty()));
  }

  @Test
  void listWithTypeGlobalReturnsMultipleGlobalWebhooks() {
    Webhook globalWebhook1 = createGlobalWebhook("globalHook1");
    Webhook globalWebhook2 = createGlobalWebhook("globalHook2");
    Webhook repositoryWebhook = createRepositoryWebhook("repoHook");

    when(webhookService.getWebhooks()).thenReturn(Arrays.asList(globalWebhook1, repositoryWebhook, globalWebhook2));

    List<ReferenceXO> result = underTest.listWithTypeGlobal();

    assertThat(result, hasSize(2));
    assertThat(result.get(0).getId(), is("globalHook1"));
    assertThat(result.get(1).getId(), is("globalHook2"));
  }

  private static Webhook createGlobalWebhook(final String name) {
    return new GlobalWebhook()
    {
      @Override
      public String getName() {
        return name;
      }
    };
  }

  private static Webhook createRepositoryWebhook(final String name) {
    return new RepositoryWebhook()
    {
      @Override
      public String getName() {
        return name;
      }
    };
  }
}
