package org.globalart.astrum.notifications.internal;

import java.util.ArrayList;
import java.util.List;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * A chat that receives repository notifications: a Telegram chat reached through a bot, or an incoming webhook of
 * Slack, Mattermost or Rocket.Chat. The bot token or the webhook URL lives in the secrets store under
 * {@link #getSecretId()}.
 *
 * <p>
 * Messages go out with the global HTTP settings ({@link #PROXY_GLOBAL}, also the mode of channels stored before the
 * connection could be chosen), directly ({@link #PROXY_NONE}), or through a proxy of the channel's own
 * ({@link #PROXY_CUSTOM}); the password of that proxy lives in the secrets store under
 * {@link #getProxyPasswordSecretId()}.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NotificationChannel
{
  public static final String TELEGRAM = "telegram";

  public static final String SLACK = "slack";

  public static final String RUSSIAN = "ru";

  public static final String ENGLISH = "en";

  public static final String PROXY_GLOBAL = "global";

  public static final String PROXY_NONE = "none";

  public static final String PROXY_CUSTOM = "custom";

  public static final String PROXY_HTTP = "http";

  public static final String PROXY_SOCKS5 = "socks5";

  private String id;

  private String name;

  private String type;

  private boolean enabled = true;

  private String chatId;

  private Long messageThreadId;

  private String secretId;

  private String targetHost;

  private List<String> repositories = new ArrayList<>();

  private List<String> events = new ArrayList<>();

  private String language = RUSSIAN;

  private String proxyMode = PROXY_GLOBAL;

  private String proxyType;

  private String proxyHost;

  private Integer proxyPort;

  private String proxyUsername;

  private String proxyPasswordSecretId;

  public NotificationChannel copy() {
    NotificationChannel copy = new NotificationChannel();
    copy.id = id;
    copy.name = name;
    copy.type = type;
    copy.enabled = enabled;
    copy.chatId = chatId;
    copy.messageThreadId = messageThreadId;
    copy.secretId = secretId;
    copy.targetHost = targetHost;
    copy.repositories = new ArrayList<>(repositories);
    copy.events = new ArrayList<>(events);
    copy.language = language;
    copy.proxyMode = proxyMode;
    copy.proxyType = proxyType;
    copy.proxyHost = proxyHost;
    copy.proxyPort = proxyPort;
    copy.proxyUsername = proxyUsername;
    copy.proxyPasswordSecretId = proxyPasswordSecretId;
    return copy;
  }

  @JsonIgnore
  public boolean isTelegram() {
    return TELEGRAM.equals(type);
  }

  @JsonIgnore
  public boolean isEnglish() {
    return ENGLISH.equals(language);
  }

  /**
   * @return whether the messages go through a proxy of the channel's own
   */
  @JsonIgnore
  public boolean isCustomProxy() {
    return PROXY_CUSTOM.equals(proxyMode);
  }

  /**
   * @return whether the messages go straight to the chat, past the proxy of the global HTTP settings
   */
  @JsonIgnore
  public boolean isDirect() {
    return PROXY_NONE.equals(proxyMode);
  }

  @JsonIgnore
  public boolean isSocksProxy() {
    return isCustomProxy() && PROXY_SOCKS5.equals(proxyType);
  }

  public boolean wants(final NoticeAction action) {
    return events.contains(action.id());
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  public String getChatId() {
    return chatId;
  }

  public void setChatId(final String chatId) {
    this.chatId = chatId;
  }

  public Long getMessageThreadId() {
    return messageThreadId;
  }

  public void setMessageThreadId(final Long messageThreadId) {
    this.messageThreadId = messageThreadId;
  }

  public String getSecretId() {
    return secretId;
  }

  public void setSecretId(final String secretId) {
    this.secretId = secretId;
  }

  public String getTargetHost() {
    return targetHost;
  }

  public void setTargetHost(final String targetHost) {
    this.targetHost = targetHost;
  }

  public List<String> getRepositories() {
    return repositories;
  }

  public void setRepositories(final List<String> repositories) {
    this.repositories = repositories != null ? new ArrayList<>(repositories) : new ArrayList<>();
  }

  public List<String> getEvents() {
    return events;
  }

  public void setEvents(final List<String> events) {
    this.events = events != null ? new ArrayList<>(events) : new ArrayList<>();
  }

  public String getLanguage() {
    return language;
  }

  public void setLanguage(final String language) {
    this.language = language;
  }

  public String getProxyMode() {
    return proxyMode;
  }

  public void setProxyMode(final String proxyMode) {
    this.proxyMode = proxyMode != null ? proxyMode : PROXY_GLOBAL;
  }

  public String getProxyType() {
    return proxyType;
  }

  public void setProxyType(final String proxyType) {
    this.proxyType = proxyType;
  }

  public String getProxyHost() {
    return proxyHost;
  }

  public void setProxyHost(final String proxyHost) {
    this.proxyHost = proxyHost;
  }

  public Integer getProxyPort() {
    return proxyPort;
  }

  public void setProxyPort(final Integer proxyPort) {
    this.proxyPort = proxyPort;
  }

  public String getProxyUsername() {
    return proxyUsername;
  }

  public void setProxyUsername(final String proxyUsername) {
    this.proxyUsername = proxyUsername;
  }

  public String getProxyPasswordSecretId() {
    return proxyPasswordSecretId;
  }

  public void setProxyPasswordSecretId(final String proxyPasswordSecretId) {
    this.proxyPasswordSecretId = proxyPasswordSecretId;
  }

  @Override
  public String toString() {
    return "NotificationChannel{id=" + id + ", name=" + name + ", type=" + type + ", enabled=" + enabled + '}';
  }
}
