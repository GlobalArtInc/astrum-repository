
package org.globalart.astrum.security;

import java.util.Objects;

/**
 * Client info about WHO is doing something.
 */
public class ClientInfo
{
  private final String userid;

  private final String remoteIP;

  private final String userAgent;

  private final String path;

  private ClientInfo(Builder builder) {
    this.userid = builder.userId;
    this.remoteIP = builder.remoteIP;
    this.userAgent = builder.userAgent;
    this.path = builder.path;
  }

  public String getRemoteIP() {
    return remoteIP;
  }

  public String getUserid() {
    return userid;
  }

  public String getUserAgent() {
    return userAgent;
  }

  public String getPath() {
    return path;
  }

  public static Builder builder() {
    return new Builder();
  }

  public static class Builder
  {
    private String userId;

    private String remoteIP;

    private String userAgent;

    private String path;

    public Builder userId(final String userId) {
      this.userId = userId;
      return this;
    }

    public Builder remoteIP(final String remoteIP) {
      this.remoteIP = remoteIP;
      return this;
    }

    public Builder userAgent(final String userAgent) {
      this.userAgent = userAgent;
      return this;
    }

    public Builder path(final String path) {
      this.path = path;
      return this;
    }

    public ClientInfo build() {
      return new ClientInfo(this);
    }
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ClientInfo that = (ClientInfo) o;
    return Objects.equals(userid, that.userid) &&
        Objects.equals(remoteIP, that.remoteIP) &&
        Objects.equals(userAgent, that.userAgent) &&
        Objects.equals(path, that.path);
  }

  @Override
  public int hashCode() {
    return Objects.hash(userid, remoteIP, userAgent, path);
  }

  @Override
  public String toString() {
    return "ClientInfo{" +
        "userid='" + userid + '\'' +
        ", remoteIP='" + remoteIP + '\'' +
        ", userAgent='" + userAgent + '\'' +
        ", path='" + path + '\'' +
        '}';
  }
}
