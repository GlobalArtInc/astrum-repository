package org.globalart.astrum.notifications.internal;

import java.util.ArrayList;
import java.util.List;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.lessThanOrEqualTo;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.startsWith;

public class NotificationMessagesTest
{
  private static NotificationChannel channel(final String type, final String language) {
    NotificationChannel channel = new NotificationChannel();
    channel.setId("c1");
    channel.setName("Releases <team>");
    channel.setType(type);
    channel.setLanguage(language);
    return channel;
  }

  private static ComponentNotice maven(final NoticeAction action) {
    return new ComponentNotice("maven-releases", "maven2", "org.example", "demo", "1.0.0", action, "roman",
        "org/example/demo/1.0.0", null, "https://repo.example.com/");
  }

  @Test
  public void russianTelegramMessageOfPublishedComponent() {
    String text = NotificationMessages.single(channel("telegram", "ru"), maven(NoticeAction.PUBLISHED));

    assertThat(text, is("📦 <b>Опубликован компонент</b>\n"
        + "<b>org.example:demo:1.0.0</b>\n"
        + "Репозиторий: <code>maven-releases</code> · maven2\n"
        + "Автор: roman\n"
        + "<a href=\"https://repo.example.com/#browse/browse:maven-releases:org%2Fexample%2Fdemo%2F1.0.0\">"
        + "Открыть в Astrum Repository</a>"));
  }

  @Test
  public void englishSlackMessageOfUpdatedComponent() {
    String text = NotificationMessages.single(channel("slack", "en"), maven(NoticeAction.UPDATED));

    assertThat(text, is("🔄 *Component updated*\n"
        + "`org.example:demo:1.0.0`\n"
        + "Repository: `maven-releases` · maven2\n"
        + "By: roman\n"
        + "<https://repo.example.com/#browse/browse:maven-releases:org%2Fexample%2Fdemo%2F1.0.0"
        + "|Open in Astrum Repository>"));
  }

  @Test
  public void deletedComponentHasNoLink() {
    String text = NotificationMessages.single(channel("telegram", "en"), maven(NoticeAction.DELETED));

    assertThat(text, startsWith("🗑️ <b>Component deleted</b>"));
    assertThat(text, not(containsString("href")));
  }

  @Test
  public void escapesHtmlInNamesAndUsers() {
    ComponentNotice notice = new ComponentNotice("raw-hosted", "raw", "/docs", "docs/<script>&\"x\".txt", "",
        NoticeAction.PUBLISHED, "eve<b>", null, null, null);

    String text = NotificationMessages.single(channel("telegram", "en"), notice);

    assertThat(text, containsString("<b>docs/&lt;script&gt;&amp;&quot;x&quot;.txt</b>"));
    assertThat(text, containsString("By: eve&lt;b&gt;"));
    assertThat(text, not(containsString("<script>")));
  }

  @Test
  public void escapesMrkdwnControlCharacters() {
    ComponentNotice notice = new ComponentNotice("raw-hosted", "raw", "", "a<b>&`c`", "", NoticeAction.PUBLISHED,
        null, null, null, null);

    String text = NotificationMessages.single(channel("slack", "en"), notice);

    assertThat(text, containsString("`a&lt;b&gt;&amp;'c'`"));
  }

  @Test
  public void dockerImageIsWrittenWithTagAndPullCommand() {
    ComponentNotice notice = new ComponentNotice("docker-hosted", "docker", "", "team/app", "1.4", NoticeAction.PUBLISHED,
        "ci", "v2/team/app/tags/1.4", "repo.example.com:8085", "https://repo.example.com");

    String text = NotificationMessages.single(channel("telegram", "en"), notice);

    assertThat(text, startsWith("🐳 <b>Component published</b>\n<b>team/app:1.4</b>\n"));
    assertThat(text, containsString("<code>docker pull repo.example.com:8085/team/app:1.4</code>"));
  }

  @Test
  public void coordinatesFollowTheFormat() {
    assertThat(NotificationMessages.coordinates(maven(NoticeAction.PUBLISHED)), is("org.example:demo:1.0.0"));
    assertThat(NotificationMessages.coordinates(
        new ComponentNotice("npm", "npm", "scope", "pkg", "2.0.1", NoticeAction.PUBLISHED, null, null, null, null)),
        is("@scope/pkg@2.0.1"));
    assertThat(NotificationMessages.coordinates(
        new ComponentNotice("npm", "npm", "", "left-pad", "1.3.0", NoticeAction.PUBLISHED, null, null, null, null)),
        is("left-pad@1.3.0"));
    assertThat(NotificationMessages.coordinates(
        new ComponentNotice("raw", "raw", "/dir", "dir/file.txt", "", NoticeAction.PUBLISHED, null, null, null, null)),
        is("dir/file.txt"));
    assertThat(NotificationMessages.coordinates(
        new ComponentNotice("pypi", "pypi", "", "requests", "2.31.0", NoticeAction.PUBLISHED, null, null, null, null)),
        is("requests:2.31.0"));
  }

  @Test
  public void linkPointsToTheRepositoryWhenTheNodeIsUnknown() {
    ComponentNotice notice = new ComponentNotice("my repo", "raw", "", "a", "", NoticeAction.UPDATED, null, null, null,
        "http://host:8081/");

    assertThat(NotificationMessages.link(notice), is("http://host:8081/#browse/browse:my%20repo"));
    assertThat(NotificationMessages.link(notice.withAction(NoticeAction.DELETED)), is(nullValue()));
    assertThat(NotificationMessages.link(new ComponentNotice("r", "raw", "", "a", "", NoticeAction.UPDATED, null, null,
        null, null)), is(nullValue()));
  }

  @Test
  public void systemUserIsNamedAndUnknownUserIsLeftOut() {
    ComponentNotice purged = new ComponentNotice("maven-releases", "maven2", "g", "a", "1", NoticeAction.DELETED,
        "*SYSTEM", null, null, null);
    ComponentNotice unknown = new ComponentNotice("maven-releases", "maven2", "g", "a", "1", NoticeAction.DELETED,
        "*UNKNOWN", null, null, null);

    assertThat(NotificationMessages.single(channel("telegram", "ru"), purged), containsString("Автор: система"));
    assertThat(NotificationMessages.single(channel("telegram", "ru"), unknown), not(containsString("Автор")));
  }

  @Test
  public void upToFiveChangesAreSentOneByOne() {
    List<ComponentNotice> notices = notices(5);

    List<String> messages = NotificationMessages.render(channel("telegram", "en"), notices, 0);

    assertThat(messages, hasSize(5));
    assertThat(messages.get(4), containsString("<b>org.example:lib-4:1.0</b>"));
  }

  @Test
  public void moreThanFiveChangesBecomeOneSummary() {
    List<String> messages = NotificationMessages.render(channel("telegram", "ru"), notices(6), 0);

    assertThat(messages, hasSize(1));
    String summary = messages.get(0);
    assertThat(summary, startsWith("📋 <b>Изменения компонентов: 6</b>\n"));
    assertThat(summary, containsString("📦 <b>org.example:lib-0:1.0</b> · <code>maven-releases</code>"));
    assertThat(summary, containsString("📦 <b>org.example:lib-5:1.0</b>"));
    assertThat(summary, containsString("Автор: roman"));
  }

  @Test
  public void longSummaryIsTruncatedAndCountsTheRest() {
    List<String> messages = NotificationMessages.render(channel("slack", "en"), notices(30), 7);

    String summary = messages.get(0);
    assertThat(summary, startsWith("📋 *Component changes: 37*\n"));
    assertThat(summary, containsString("`org.example:lib-19:1.0`"));
    assertThat(summary, not(containsString("lib-20:")));
    assertThat(summary, containsString("…and 17 more"));
    assertThat(summary.length(), lessThanOrEqualTo(NotificationMessages.SUMMARY_MAX_LENGTH + 100));
  }

  @Test
  public void summaryOmitsTheUserWhenSeveralUsersChanged() {
    List<ComponentNotice> notices = notices(6);
    notices.set(3, new ComponentNotice("maven-releases", "maven2", "org.example", "other", "1.0",
        NoticeAction.DELETED, "alice", null, null, null));

    String summary = NotificationMessages.render(channel("telegram", "en"), notices, 0).get(0);

    assertThat(summary, containsString("🗑️ <b>org.example:other:1.0</b>"));
    assertThat(summary, not(containsString("By:")));
  }

  @Test
  public void testMessageNamesTheChannel() {
    assertThat(NotificationMessages.test(channel("telegram", "en"), "https://repo.example.com"),
        is("✅ <b>Test message from Astrum Repository</b>\n"
            + "Notifications of the channel “Releases &lt;team&gt;” are set up correctly.\n"
            + "<a href=\"https://repo.example.com\">Astrum Repository</a>"));
    assertThat(NotificationMessages.test(channel("slack", "ru"), null),
        is("✅ *Тестовое сообщение от Astrum Repository*\nУведомления канала «Releases &lt;team&gt;» настроены правильно."));
  }

  private static List<ComponentNotice> notices(final int count) {
    List<ComponentNotice> notices = new ArrayList<>();
    for (int i = 0; i < count; i++) {
      notices.add(new ComponentNotice("maven-releases", "maven2", "org.example", "lib-" + i, "1.0",
          NoticeAction.PUBLISHED, "roman", null, null, "https://repo.example.com"));
    }
    return notices;
  }
}
