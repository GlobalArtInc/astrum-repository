package org.globalart.astrum.notifications.internal;

import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.GenericEntity;

import org.globalart.astrum.common.app.BaseUrlManager;
import org.globalart.astrum.notifications.internal.NotificationChannelStore.SecretChange;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.rest.ValidationErrorXO;

import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.util.ThreadContext;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class NotificationChannelsResourceTest
{
  private static final String TOKEN = "123456789:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsaw";

  private NotificationChannelStore store;

  private NotificationTransport transport;

  private RepositoryManager repositoryManager;

  private NotificationChannelsResource resource;

  private final List<NotificationChannel> channels = new ArrayList<>();

  private Subject subject;

  @Before
  public void setUp() {
    subject = mock(Subject.class);
    when(subject.isAuthenticated()).thenReturn(true);
    when(subject.getPrincipal()).thenReturn("admin");
    ThreadContext.bind(subject);
    store = mock(NotificationChannelStore.class);
    transport = mock(NotificationTransport.class);
    repositoryManager = mock(RepositoryManager.class);
    BaseUrlManager baseUrlManager = mock(BaseUrlManager.class);
    when(baseUrlManager.getOrDetect()).thenReturn("https://repo.example.com");
    when(repositoryManager.get("maven-releases")).thenReturn(mock(Repository.class));
    when(store.list()).thenAnswer(invocation -> channels.stream().map(NotificationChannel::copy).toList());
    when(store.find(anyString())).thenAnswer(invocation -> channels.stream()
        .filter(channel -> channel.getId().equals(invocation.getArgument(0)))
        .findFirst()
        .map(NotificationChannel::copy));
    when(store.create(any(), anyString(), any(), any())).thenAnswer(invocation -> {
      NotificationChannel channel = ((NotificationChannel) invocation.getArgument(0)).copy();
      channel.setId("new");
      channel.setSecretId("_secret");
      channel.setProxyPasswordSecretId(invocation.getArgument(2) != null ? "_proxy" : null);
      return channel;
    });
    when(store.update(any(), any(), any(), any())).thenAnswer(invocation -> {
      NotificationChannel channel = ((NotificationChannel) invocation.getArgument(0)).copy();
      channel.setSecretId("_secret");
      return channel;
    });
    resource = new NotificationChannelsResource(store, transport, repositoryManager, baseUrlManager);
  }

  @After
  public void tearDown() {
    ThreadContext.unbindSubject();
  }

  private static NotificationChannelXO telegramXO() {
    NotificationChannelXO xo = new NotificationChannelXO();
    xo.setName("Releases");
    xo.setType("telegram");
    xo.setChatId("-1001234567890");
    xo.setSecret(TOKEN);
    xo.setEvents(List.of("deleted", "published"));
    xo.setRepositories(List.of("maven-releases"));
    xo.setLanguage("en");
    return xo;
  }

  private static NotificationChannelXO slackXO() {
    NotificationChannelXO xo = new NotificationChannelXO();
    xo.setName("Builds");
    xo.setType("slack");
    xo.setSecret("https://hooks.slack.com/services/T000/B000/XXXX");
    xo.setEvents(List.of("published"));
    return xo;
  }

  private NotificationChannel saved(final String id, final String type) {
    NotificationChannel channel = new NotificationChannel();
    channel.setId(id);
    channel.setName("Saved " + id);
    channel.setType(type);
    channel.setChatId("telegram".equals(type) ? "-100" : null);
    channel.setTargetHost("slack".equals(type) ? "hooks.slack.com" : null);
    channel.setSecretId("_stored");
    channel.setEvents(List.of("published"));
    channels.add(channel);
    return channel;
  }

  private static NotificationChannelXO proxyXO() {
    NotificationChannelXO xo = telegramXO();
    xo.setProxyMode("custom");
    xo.setProxyType("socks5");
    xo.setProxyHost(" proxy.internal ");
    xo.setProxyPort(1080);
    xo.setProxyUsername("robot");
    xo.setProxyPassword(" pass word ");
    return xo;
  }

  private NotificationChannel savedWithProxy(final String id) {
    NotificationChannel channel = saved(id, "telegram");
    channel.setName("Releases");
    channel.setProxyMode("custom");
    channel.setProxyType("socks5");
    channel.setProxyHost("proxy.internal");
    channel.setProxyPort(1080);
    channel.setProxyUsername("robot");
    channel.setProxyPasswordSecretId("_proxy");
    return channel;
  }

  private SecretChange updatedProxyPassword(final String id, final NotificationChannelXO xo) {
    resource.update(id, xo);
    ArgumentCaptor<SecretChange> change = ArgumentCaptor.forClass(SecretChange.class);
    verify(store).update(any(), any(), change.capture(), any());
    return change.getValue();
  }

  private static NotificationSecrets withSecret(final String secret) {
    return argThat(secrets -> secrets != null && secrets.secret().equals(secret));
  }

  private static void assertBadRequest(final Runnable call, final String field, final String message) {
    WebApplicationException e = assertThrows(WebApplicationException.class, call::run);
    assertThat(e.getResponse().getStatus(), is(400));
    Object entity = e.getResponse().getEntity();
    ValidationErrorXO error =
        (ValidationErrorXO) (entity instanceof GenericEntity<?> generic ? generic.getEntity() : entity);
    assertThat(error.getId(), is(field));
    assertThat(error.getMessage(), containsString(message));
  }

  @Test
  public void createsATelegramChannel() {
    NotificationChannelXO created = resource.create(telegramXO());

    ArgumentCaptor<NotificationChannel> channel = ArgumentCaptor.forClass(NotificationChannel.class);
    verify(store).create(channel.capture(), eq(TOKEN), any(), any());
    assertThat(channel.getValue().getEvents(), contains("published", "deleted"));
    assertThat(channel.getValue().getLanguage(), is("en"));
    assertThat(created.getSecret(), is(PasswordPlaceholder.get()));
    assertThat(created.getTarget(), is("-1001234567890"));
  }

  @Test
  public void createsASlackChannelAndShowsOnlyTheWebhookHost() {
    NotificationChannelXO xo = slackXO();
    xo.setLanguage(null);

    NotificationChannelXO created = resource.create(xo);

    assertThat(created.getTarget(), is("hooks.slack.com"));
    assertThat(created.getLanguage(), is("ru"));
    assertThat(created.getSecret(), is(PasswordPlaceholder.get()));
    assertThat(created.getChatId(), is(nullValue()));
  }

  @Test
  public void listNeverReturnsSecrets() {
    saved("a", "slack");

    NotificationChannelXO xo = resource.list().get(0);

    assertThat(xo.getSecret(), is(PasswordPlaceholder.get()));
    assertThat(xo.getTarget(), is("hooks.slack.com"));
  }

  @Test
  public void rejectsInvalidChannels() {
    NotificationChannelXO noName = telegramXO();
    noName.setName("  ");
    assertBadRequest(() -> resource.create(noName), "name", "Name is required");

    NotificationChannelXO badType = telegramXO();
    badType.setType("email");
    assertBadRequest(() -> resource.create(badType), "type", "telegram or slack");

    NotificationChannelXO noChat = telegramXO();
    noChat.setChatId(null);
    assertBadRequest(() -> resource.create(noChat), "chatId", "Chat ID is required");

    NotificationChannelXO badChat = telegramXO();
    badChat.setChatId("12; DROP");
    assertBadRequest(() -> resource.create(badChat), "chatId", "Chat ID must be");

    NotificationChannelXO badThread = telegramXO();
    badThread.setMessageThreadId(0L);
    assertBadRequest(() -> resource.create(badThread), "messageThreadId", "positive");

    NotificationChannelXO badToken = telegramXO();
    badToken.setSecret("not-a-token/../x");
    assertBadRequest(() -> resource.create(badToken), "secret", "bot token must look like");

    NotificationChannelXO noToken = telegramXO();
    noToken.setSecret("");
    assertBadRequest(() -> resource.create(noToken), "secret", "Bot token is required");

    NotificationChannelXO badUrl = slackXO();
    badUrl.setSecret("ftp://hooks.example.com/x");
    assertBadRequest(() -> resource.create(badUrl), "secret", "http or https URL");

    NotificationChannelXO noEvents = telegramXO();
    noEvents.setEvents(List.of());
    assertBadRequest(() -> resource.create(noEvents), "events", "at least one event");

    NotificationChannelXO badEvent = telegramXO();
    badEvent.setEvents(List.of("downloaded"));
    assertBadRequest(() -> resource.create(badEvent), "events", "Unknown event");

    NotificationChannelXO badRepository = telegramXO();
    badRepository.setRepositories(List.of("missing"));
    assertBadRequest(() -> resource.create(badRepository), "repositories", "missing does not exist");

    NotificationChannelXO badLanguage = telegramXO();
    badLanguage.setLanguage("de");
    assertBadRequest(() -> resource.create(badLanguage), "language", "ru or en");

    verify(store, never()).create(any(), any(), any(), any());
  }

  @Test
  public void namesAreUnique() {
    saved("a", "telegram").setName("releases");
    channels.get(0).setName("releases");

    assertBadRequest(() -> resource.create(telegramXO()), "name", "already exists");
  }

  @Test
  public void updateWithThePlaceholderKeepsTheStoredSecret() {
    saved("a", "telegram");
    NotificationChannelXO xo = telegramXO();
    xo.setName("Saved a");
    xo.setSecret(PasswordPlaceholder.get());

    resource.update("a", xo);

    ArgumentCaptor<SecretChange> change = ArgumentCaptor.forClass(SecretChange.class);
    verify(store).update(any(), change.capture(), any(), any());
    assertThat(change.getValue().replace(), is(false));
  }

  @Test
  public void updateWithANewTokenReplacesTheSecret() {
    saved("a", "telegram");

    resource.update("a", telegramXO());

    ArgumentCaptor<SecretChange> change = ArgumentCaptor.forClass(SecretChange.class);
    verify(store).update(any(), change.capture(), any(), any());
    assertThat(change.getValue().replace(), is(true));
    assertThat(change.getValue().value(), is(TOKEN));
  }

  @Test
  public void changingTheTypeNeedsANewSecret() {
    saved("a", "telegram");
    NotificationChannelXO xo = slackXO();
    xo.setSecret(PasswordPlaceholder.get());

    assertBadRequest(() -> resource.update("a", xo), "secret", "Webhook URL is required");
  }

  @Test
  public void unknownChannelsAreNotFound() {
    WebApplicationException e = assertThrows(WebApplicationException.class, () -> resource.delete("missing"));
    assertThat(e.getResponse().getStatus(), is(404));
    e = assertThrows(WebApplicationException.class, () -> resource.update("missing", telegramXO()));
    assertThat(e.getResponse().getStatus(), is(404));
  }

  @Test
  public void testSendsATestMessageWithTheSavedSecret() throws Exception {
    NotificationChannel channel = saved("a", "telegram");
    when(store.secret(any())).thenReturn(TOKEN);

    NotificationTestResultXO result = resource.test("a");

    assertThat(result.isSuccess(), is(true));
    ArgumentCaptor<String> text = ArgumentCaptor.forClass(String.class);
    verify(transport).send(any(), argThat(secrets -> secrets.secret().equals(TOKEN)), text.capture());
    assertThat(text.getValue(), containsString(channel.getName()));
    assertThat(text.getValue(), containsString("https://repo.example.com"));
  }

  @Test
  public void testReportsWhatTheChatAnswered() throws Exception {
    saved("a", "telegram");
    when(store.secret(any())).thenReturn(TOKEN);
    doThrow(new NotificationDeliveryException("Telegram answered 400: Bad Request: chat not found", 400, false, 0))
        .when(transport).send(any(), any(), anyString());

    NotificationTestResultXO result = resource.test("a");

    assertThat(result.isSuccess(), is(false));
    assertThat(result.getMessage(), is("Telegram answered 400: Bad Request: chat not found"));
  }

  @Test
  public void testOfTheFormUsesTheTypedSecretOrTheSavedOne() throws Exception {
    NotificationChannelXO unsaved = slackXO();
    unsaved.setName(null);
    assertThat(resource.testUnsaved(unsaved).isSuccess(), is(true));
    verify(transport).send(any(), withSecret("https://hooks.slack.com/services/T000/B000/XXXX"), anyString());

    saved("a", "slack");
    when(store.secret(any())).thenReturn("https://chat.example.com/hooks/abc");
    NotificationChannelXO edited = slackXO();
    edited.setId("a");
    edited.setSecret(PasswordPlaceholder.get());
    assertThat(resource.testUnsaved(edited).isSuccess(), is(true));
    verify(transport).send(any(), withSecret("https://chat.example.com/hooks/abc"), anyString());

    NotificationChannelXO withoutSecret = telegramXO();
    withoutSecret.setSecret(PasswordPlaceholder.get());
    assertBadRequest(() -> resource.testUnsaved(withoutSecret), "secret", "Bot token is required");
  }

  @Test
  public void usersWhoMayOnlyReadSettingsCannotChangeChannels() {
    doThrow(new AuthorizationException("denied")).when(subject).checkPermission("astrum:settings:update");
    saved("a", "telegram");

    assertThat(resource.list().size(), is(1));
    assertThrows(AuthorizationException.class, () -> resource.create(telegramXO()));
    assertThrows(AuthorizationException.class, () -> resource.update("a", telegramXO()));
    assertThrows(AuthorizationException.class, () -> resource.delete("a"));
    assertThrows(AuthorizationException.class, () -> resource.test("a"));
    assertThrows(AuthorizationException.class, () -> resource.testUnsaved(telegramXO()));
    verify(store, never()).delete(anyString());
  }

  @Test
  public void readingNeedsSettingsReadAndChangingNeedsSettingsUpdate() throws Exception {
    assertPermission("list", "astrum:settings:read");
    assertPermission("create", "astrum:settings:update", NotificationChannelXO.class);
    assertPermission("update", "astrum:settings:update", String.class, NotificationChannelXO.class);
    assertPermission("delete", "astrum:settings:update", String.class);
    assertPermission("test", "astrum:settings:update", String.class);
    assertPermission("testUnsaved", "astrum:settings:update", NotificationChannelXO.class);
  }

  private static void assertPermission(final String name, final String permission, final Class<?>... parameters)
      throws Exception
  {
    Method method = NotificationChannelsResource.class.getMethod(name, parameters);
    assertThat(method.getAnnotation(RequiresAuthentication.class), is(notNullValue()));
    assertThat(method.getAnnotation(RequiresPermissions.class).value(), is(new String[]{permission}));
  }

  @Test
  public void storedChannelsWithoutSecretsHaveNoPlaceholder() {
    NotificationChannel channel = saved("a", "telegram");
    channel.setSecretId(null);

    assertThat(NotificationChannelsResource.toXO(channel).getSecret(), is(""));
    assertThat(Optional.ofNullable(NotificationChannelsResource.toXO(channel).getTarget()), is(Optional.of("-100")));
  }

  @Test
  public void createsAChannelWithItsOwnProxy() {
    NotificationChannelXO created = resource.create(proxyXO());

    ArgumentCaptor<NotificationChannel> channel = ArgumentCaptor.forClass(NotificationChannel.class);
    verify(store).create(channel.capture(), eq(TOKEN), eq(" pass word "), any());
    assertThat(channel.getValue().getProxyMode(), is("custom"));
    assertThat(channel.getValue().getProxyType(), is("socks5"));
    assertThat(channel.getValue().getProxyHost(), is("proxy.internal"));
    assertThat(channel.getValue().getProxyPort(), is(1080));
    assertThat(channel.getValue().getProxyUsername(), is("robot"));
    assertThat(created.getProxyPassword(), is(PasswordPlaceholder.get()));
    assertThat(created.getProxyHost(), is("proxy.internal"));
  }

  @Test
  public void channelsWithoutAConnectionUseTheGlobalSettingsAndForgetProxyFields() {
    NotificationChannelXO legacy = telegramXO();
    assertThat(resource.create(legacy).getProxyMode(), is("global"));

    NotificationChannelXO direct = proxyXO();
    direct.setName("Direct");
    direct.setProxyMode("none");
    NotificationChannelXO created = resource.create(direct);

    ArgumentCaptor<NotificationChannel> channel = ArgumentCaptor.forClass(NotificationChannel.class);
    verify(store, times(2)).create(channel.capture(), eq(TOKEN), eq(null), any());
    assertThat(channel.getValue().getProxyMode(), is("none"));
    assertThat(channel.getValue().getProxyHost(), is(nullValue()));
    assertThat(channel.getValue().getProxyUsername(), is(nullValue()));
    assertThat(created.getProxyPassword(), is(""));
  }

  @Test
  public void rejectsInvalidProxies() {
    NotificationChannelXO badMode = proxyXO();
    badMode.setProxyMode("system");
    assertBadRequest(() -> resource.create(badMode), "proxyMode", "global, none or custom");

    NotificationChannelXO badType = proxyXO();
    badType.setProxyType("socks4");
    assertBadRequest(() -> resource.create(badType), "proxyType", "http or socks5");

    NotificationChannelXO noHost = proxyXO();
    noHost.setProxyHost(" ");
    assertBadRequest(() -> resource.create(noHost), "proxyHost", "Proxy host is required");

    NotificationChannelXO url = proxyXO();
    url.setProxyHost("http://proxy.internal:3128");
    assertBadRequest(() -> resource.create(url), "proxyHost", "without a scheme or port");

    NotificationChannelXO longHost = proxyXO();
    longHost.setProxyHost("a".repeat(256));
    assertBadRequest(() -> resource.create(longHost), "proxyHost", "255 characters");

    NotificationChannelXO noPort = proxyXO();
    noPort.setProxyPort(null);
    assertBadRequest(() -> resource.create(noPort), "proxyPort", "Proxy port is required");

    NotificationChannelXO zeroPort = proxyXO();
    zeroPort.setProxyPort(0);
    assertBadRequest(() -> resource.create(zeroPort), "proxyPort", "between 1 and 65535");

    NotificationChannelXO bigPort = proxyXO();
    bigPort.setProxyPort(65536);
    assertBadRequest(() -> resource.create(bigPort), "proxyPort", "between 1 and 65535");

    NotificationChannelXO longUser = proxyXO();
    longUser.setProxyUsername("u".repeat(256));
    assertBadRequest(() -> resource.create(longUser), "proxyUsername", "255 characters");

    NotificationChannelXO longPassword = proxyXO();
    longPassword.setProxyPassword("p".repeat(256));
    assertBadRequest(() -> resource.create(longPassword), "proxyPassword", "255 characters");

    NotificationChannelXO passwordWithoutUser = proxyXO();
    passwordWithoutUser.setProxyUsername(null);
    assertBadRequest(() -> resource.create(passwordWithoutUser), "proxyUsername", "username that goes with");

    verify(store, never()).create(any(), any(), any(), any());
  }

  @Test
  public void acceptsProxyAddressesAndOptionalCredentials() {
    NotificationChannelXO ipv6 = proxyXO();
    ipv6.setProxyHost("[fd00::1]");
    ipv6.setProxyUsername(null);
    ipv6.setProxyPassword("");
    ipv6.setProxyType("http");
    assertThat(resource.create(ipv6).getProxyHost(), is("fd00::1"));

    NotificationChannelXO userOnly = proxyXO();
    userOnly.setName("User only");
    userOnly.setProxyHost("10.0.0.5");
    userOnly.setProxyPassword(null);
    assertThat(resource.create(userOnly).getProxyUsername(), is("robot"));
  }

  @Test
  public void thePlaceholderKeepsTheStoredProxyPassword() {
    savedWithProxy("a");
    NotificationChannelXO xo = proxyXO();
    xo.setProxyHost("PROXY.internal");
    xo.setProxyPassword(PasswordPlaceholder.get());

    assertThat(updatedProxyPassword("a", xo).replace(), is(false));
  }

  @Test
  public void aNewProxyPasswordReplacesTheStoredOne() {
    savedWithProxy("a");

    SecretChange change = updatedProxyPassword("a", proxyXO());

    assertThat(change.replace(), is(true));
    assertThat(change.value(), is(" pass word "));
  }

  @Test
  public void anEmptyProxyPasswordRemovesTheStoredOne() {
    savedWithProxy("a");
    NotificationChannelXO xo = proxyXO();
    xo.setProxyPassword("");

    SecretChange change = updatedProxyPassword("a", xo);

    assertThat(change.replace(), is(true));
    assertThat(change.value(), is(nullValue()));
  }

  @Test
  public void leavingTheProxyOrItsUsernameRemovesTheStoredPassword() {
    savedWithProxy("a");
    NotificationChannelXO global = proxyXO();
    global.setProxyMode("global");
    global.setProxyPassword(PasswordPlaceholder.get());
    assertThat(updatedProxyPassword("a", global), is(SecretChange.clear()));

    savedWithProxy("b");
    NotificationChannelXO anonymous = proxyXO();
    anonymous.setName("Saved b");
    anonymous.setProxyUsername("");
    anonymous.setProxyPassword(PasswordPlaceholder.get());
    resource.update("b", anonymous);
    ArgumentCaptor<SecretChange> change = ArgumentCaptor.forClass(SecretChange.class);
    verify(store, times(2)).update(any(), any(), change.capture(), any());
    assertThat(change.getValue(), is(SecretChange.clear()));
  }

  @Test
  public void theStoredProxyPasswordIsNeverSentToAnotherProxy() {
    savedWithProxy("a");
    NotificationChannelXO otherHost = proxyXO();
    otherHost.setProxyHost("attacker.example");
    otherHost.setProxyPassword(PasswordPlaceholder.get());
    assertBadRequest(() -> resource.update("a", otherHost), "proxyPassword", "Enter the proxy password again");

    NotificationChannelXO otherPort = proxyXO();
    otherPort.setProxyPort(1081);
    otherPort.setProxyPassword(null);
    assertBadRequest(() -> resource.update("a", otherPort), "proxyPassword", "Enter the proxy password again");

    NotificationChannelXO otherType = proxyXO();
    otherType.setId("a");
    otherType.setProxyType("http");
    otherType.setProxyPassword(PasswordPlaceholder.get());
    assertBadRequest(() -> resource.testUnsaved(otherType), "proxyPassword", "Enter the proxy password again");
  }

  @Test
  public void listNeverReturnsProxyPasswords() {
    savedWithProxy("a");

    NotificationChannelXO xo = resource.list().get(0);

    assertThat(xo.getProxyPassword(), is(PasswordPlaceholder.get()));
    assertThat(xo.getProxyMode(), is("custom"));
    assertThat(xo.getProxyType(), is("socks5"));
    assertThat(xo.getProxyPort(), is(1080));
  }

  @Test
  public void testMessagesUseTheProxyPasswordOfTheFormOrTheSavedOne() throws Exception {
    savedWithProxy("a");
    when(store.secret(any())).thenReturn(TOKEN);
    when(store.proxyPassword(any())).thenReturn("stored");

    assertThat(resource.test("a").isSuccess(), is(true));
    verify(transport).send(any(), argThat(secrets -> "stored".equals(secrets.proxyPassword())), anyString());

    NotificationChannelXO typed = proxyXO();
    typed.setId("a");
    resource.testUnsaved(typed);
    verify(transport).send(any(), argThat(secrets -> " pass word ".equals(secrets.proxyPassword())), anyString());

    NotificationChannelXO kept = proxyXO();
    kept.setId("a");
    kept.setSecret(PasswordPlaceholder.get());
    kept.setProxyPassword(PasswordPlaceholder.get());
    resource.testUnsaved(kept);
    ArgumentCaptor<NotificationChannel> channel = ArgumentCaptor.forClass(NotificationChannel.class);
    verify(transport, times(2))
        .send(channel.capture(), argThat(secrets -> "stored".equals(secrets.proxyPassword())), anyString());
    assertThat(channel.getValue().getProxyHost(), is("proxy.internal"));
    assertThat(channel.getValue().isSocksProxy(), is(true));

    NotificationChannelXO cleared = proxyXO();
    cleared.setId("a");
    cleared.setProxyPassword("");
    resource.testUnsaved(cleared);
    verify(transport).send(any(), argThat(secrets -> TOKEN.equals(secrets.secret()) && secrets.proxyPassword() == null),
        anyString());
  }
}
