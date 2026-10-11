package org.globalart.astrum.notifications.internal;

import java.net.URLEncoder;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import javax.annotation.Nullable;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Writes notification messages: HTML for Telegram, mrkdwn for Slack-compatible webhooks and the text of JSON
 * webhooks, in Russian or English.
 */
public final class NotificationMessages
{
  /**
   * More changes than this in one window of a channel are told in one summary message.
   */
  public static final int SUMMARY_THRESHOLD = 5;

  static final int SUMMARY_MAX_LINES = 20;

  static final int SUMMARY_MAX_LENGTH = 3500;

  private static final String SYSTEM_USER = "*SYSTEM";

  private NotificationMessages() {
  }

  private record Texts(
      String published,
      String updated,
      String deleted,
      String repository,
      String by,
      String system,
      String open,
      String summary,
      String more,
      String testTitle,
      String testBody)
  {
  }

  private static final Texts RU = new Texts(
      "Опубликован компонент",
      "Обновлён компонент",
      "Удалён компонент",
      "Репозиторий",
      "Автор",
      "система",
      "Открыть в Astrum Repository",
      "Изменения компонентов: %d",
      "…и ещё %d",
      "Тестовое сообщение от Astrum Repository",
      "Уведомления канала «%s» настроены правильно.");

  private static final Texts EN = new Texts(
      "Component published",
      "Component updated",
      "Component deleted",
      "Repository",
      "By",
      "system",
      "Open in Astrum Repository",
      "Component changes: %d",
      "…and %d more",
      "Test message from Astrum Repository",
      "Notifications of the channel “%s” are set up correctly.");

  /**
   * The messages for the changes collected in one window of a channel: one message per change, or one summary when
   * there are more than {@link #SUMMARY_THRESHOLD} of them. JSON webhooks get one message per change however many
   * there are, and nothing for the changes beyond the window's capacity.
   */
  public static List<NotificationMessage> render(
      final NotificationChannel channel,
      final List<ComponentNotice> notices,
      final int overflow)
  {
    Markup markup = Markup.of(channel);
    Texts texts = texts(channel);
    if (!channel.isWebhook() && notices.size() + overflow > SUMMARY_THRESHOLD) {
      return List.of(NotificationMessage.summary(summary(markup, texts, notices, overflow)));
    }
    List<NotificationMessage> messages = new ArrayList<>();
    for (ComponentNotice notice : notices) {
      messages.add(NotificationMessage.of(notice, single(markup, texts, notice)));
    }
    return messages;
  }

  public static String single(final NotificationChannel channel, final ComponentNotice notice) {
    return single(Markup.of(channel), texts(channel), notice);
  }

  public static String test(final NotificationChannel channel, @Nullable final String baseUrl) {
    Markup markup = Markup.of(channel);
    Texts texts = texts(channel);
    StringBuilder text = new StringBuilder();
    text.append("✅ ").append(markup.bold(texts.testTitle())).append('\n');
    text.append(markup.escape(String.format(texts.testBody(), channel.getName())));
    if (baseUrl != null && !baseUrl.isBlank()) {
      text.append('\n').append(markup.link(baseUrl, "Astrum Repository"));
    }
    return text.toString();
  }

  /**
   * How the component is usually written for its format, e.g. {@code group:name:version} or {@code name:tag}.
   */
  public static String coordinates(final ComponentNotice notice) {
    String namespace = notice.namespace();
    String name = notice.name();
    String version = notice.version();
    switch (notice.format()) {
      case "docker":
        return version.isEmpty() ? name : name + ':' + version;
      case "npm": {
        String scope = namespace.isEmpty() ? "" : (namespace.startsWith("@") ? namespace : '@' + namespace) + '/';
        return scope + name + (version.isEmpty() ? "" : '@' + version);
      }
      case "raw":
        return name;
      default: {
        StringBuilder coordinates = new StringBuilder();
        if (!namespace.isEmpty()) {
          coordinates.append(namespace).append(':');
        }
        coordinates.append(name);
        if (!version.isEmpty()) {
          coordinates.append(':').append(version);
        }
        return coordinates.toString();
      }
    }
  }

  /**
   * The address of the component in the browse tree of the user interface, or of its repository when the component's
   * node is not known.
   */
  @Nullable
  public static String link(final ComponentNotice notice) {
    String baseUrl = notice.baseUrl();
    if (baseUrl == null || baseUrl.isBlank() || notice.action() == NoticeAction.DELETED) {
      return null;
    }
    StringBuilder link = new StringBuilder(baseUrl.replaceAll("/+$", ""))
        .append("/#browse/browse:")
        .append(encodeUriComponent(notice.repository()));
    if (notice.browsePath() != null && !notice.browsePath().isEmpty()) {
      link.append(':').append(encodeUriComponent(notice.browsePath()));
    }
    return link.toString();
  }

  private static String single(final Markup markup, final Texts texts, final ComponentNotice notice) {
    StringBuilder text = new StringBuilder();
    text.append(emoji(notice)).append(' ').append(markup.bold(title(texts, notice.action()))).append('\n');
    text.append(markup.strong(coordinates(notice))).append('\n');
    text.append(texts.repository()).append(": ").append(markup.code(notice.repository()));
    if (!notice.format().isEmpty()) {
      text.append(" · ").append(markup.escape(notice.format()));
    }
    String initiator = initiator(texts, notice.initiator());
    if (initiator != null) {
      text.append('\n').append(texts.by()).append(": ").append(markup.escape(initiator));
    }
    String pull = pullCommand(notice);
    if (pull != null) {
      text.append('\n').append(markup.code(pull));
    }
    String link = link(notice);
    if (link != null) {
      text.append('\n').append(markup.link(link, texts.open()));
    }
    return text.toString();
  }

  private static String summary(
      final Markup markup,
      final Texts texts,
      final List<ComponentNotice> notices,
      final int overflow)
  {
    StringBuilder text = new StringBuilder();
    text.append("📋 ").append(markup.bold(String.format(texts.summary(), notices.size() + overflow)));
    int shown = 0;
    for (ComponentNotice notice : notices) {
      String line = '\n' + emoji(notice) + ' ' + markup.strong(coordinates(notice)) + " · "
          + markup.code(notice.repository());
      if (shown == SUMMARY_MAX_LINES || text.length() + line.length() > SUMMARY_MAX_LENGTH) {
        break;
      }
      text.append(line);
      shown++;
    }
    int rest = notices.size() - shown + overflow;
    if (rest > 0) {
      text.append('\n').append(markup.escape(String.format(texts.more(), rest)));
    }
    Set<String> initiators = new HashSet<>();
    notices.forEach(notice -> initiators.add(initiator(texts, notice.initiator())));
    String initiator = initiators.size() == 1 ? initiators.iterator().next() : null;
    if (initiator != null) {
      text.append('\n').append(texts.by()).append(": ").append(markup.escape(initiator));
    }
    return text.toString();
  }

  /**
   * The command that pulls a published or updated Docker image, when the address of its registry is known.
   */
  @Nullable
  static String pullCommand(final ComponentNotice notice) {
    if (!"docker".equals(notice.format()) || notice.registry() == null || notice.action() == NoticeAction.DELETED) {
      return null;
    }
    return "docker pull " + notice.registry() + '/' + coordinates(notice);
  }

  @Nullable
  private static String initiator(final Texts texts, @Nullable final String initiator) {
    if (initiator == null || initiator.isBlank()) {
      return null;
    }
    if (SYSTEM_USER.equals(initiator)) {
      return texts.system();
    }
    if (initiator.startsWith("*")) {
      return null;
    }
    return initiator;
  }

  private static String title(final Texts texts, final NoticeAction action) {
    return switch (action) {
      case PUBLISHED -> texts.published();
      case UPDATED -> texts.updated();
      case DELETED -> texts.deleted();
    };
  }

  private static String emoji(final ComponentNotice notice) {
    return switch (notice.action()) {
      case PUBLISHED -> "docker".equals(notice.format()) ? "🐳" : "📦";
      case UPDATED -> "🔄";
      case DELETED -> "🗑️";
    };
  }

  private static Texts texts(final NotificationChannel channel) {
    return channel.isEnglish() ? EN : RU;
  }

  static String encodeUriComponent(final String value) {
    return URLEncoder.encode(value, UTF_8).replace("+", "%20");
  }

  /**
   * Formatting rules of the chat.
   */
  enum Markup
  {
    HTML {
      @Override
      String escape(final String text) {
        return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;");
      }

      @Override
      String bold(final String text) {
        return "<b>" + escape(text) + "</b>";
      }

      @Override
      String strong(final String text) {
        return bold(text);
      }

      @Override
      String code(final String text) {
        return "<code>" + escape(text) + "</code>";
      }

      @Override
      String link(final String url, final String text) {
        return "<a href=\"" + escape(url) + "\">" + escape(text) + "</a>";
      }
    },

    MRKDWN {
      @Override
      String escape(final String text) {
        return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
      }

      @Override
      String bold(final String text) {
        return "*" + escape(text) + "*";
      }

      @Override
      String strong(final String text) {
        return code(text);
      }

      @Override
      String code(final String text) {
        return "`" + escape(text.replace('`', '\'')) + "`";
      }

      @Override
      String link(final String url, final String text) {
        return "<" + escape(url).replace("|", "%7C") + "|" + escape(text) + ">";
      }
    };

    abstract String escape(String text);

    abstract String bold(String text);

    /**
     * Highlights a component name; Slack would read the underscores and asterisks of names as formatting in bold
     * text, so it gets a code span there.
     */
    abstract String strong(String text);

    abstract String code(String text);

    abstract String link(String url, String text);

    static Markup of(final NotificationChannel channel) {
      return channel.isTelegram() ? HTML : MRKDWN;
    }
  }
}
