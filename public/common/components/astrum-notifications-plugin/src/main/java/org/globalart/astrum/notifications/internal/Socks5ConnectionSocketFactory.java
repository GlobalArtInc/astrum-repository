package org.globalart.astrum.notifications.internal;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.IDN;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.Proxy;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.net.UnknownHostException;

import javax.annotation.Nullable;

import com.google.common.net.InetAddresses;
import org.apache.http.HttpHost;
import org.apache.http.conn.DnsResolver;
import org.apache.http.conn.socket.LayeredConnectionSocketFactory;
import org.apache.http.protocol.HttpContext;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.US_ASCII;
import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Connects to the target through a SOCKS5 proxy (RFC 1928), with username and password authentication (RFC 1929) when
 * a username is given. The proxy gets the target's host name and resolves it itself, which matters where the local
 * name servers filter names; together with {@link #REMOTE_DNS} the host name is never resolved here. With a factory for
 * TLS, the connection is then secured end to end with the target. The proxy's credentials belong to this factory
 * alone: neither the JVM-wide authenticator nor the {@code socksProxyHost} system properties are used.
 */
public class Socks5ConnectionSocketFactory
    implements LayeredConnectionSocketFactory
{
  /**
   * Leaves the target's host name unresolved: the address only carries the name to {@link #connectSocket}.
   */
  public static final DnsResolver REMOTE_DNS = host -> new InetAddress[]{InetAddress.getByAddress(host, new byte[4])};

  private static final int VERSION = 5;

  private static final int NO_AUTHENTICATION = 0;

  private static final int USERNAME_PASSWORD = 2;

  private static final int NO_ACCEPTABLE_METHOD = 0xFF;

  private static final int AUTHENTICATION_VERSION = 1;

  private static final int CONNECT = 1;

  private static final int IPV4 = 1;

  private static final int DOMAIN = 3;

  private static final int IPV6 = 4;

  private static final int MAX_LENGTH = 255;

  private final String proxyHost;

  private final int proxyPort;

  @Nullable
  private final String username;

  @Nullable
  private final String password;

  @Nullable
  private final LayeredConnectionSocketFactory tls;

  /**
   * @param tls secures the connection to the target, or {@code null} for plain connections
   */
  public Socks5ConnectionSocketFactory(
      final String proxyHost,
      final int proxyPort,
      @Nullable final String username,
      @Nullable final String password,
      @Nullable final LayeredConnectionSocketFactory tls)
  {
    this.proxyHost = checkNotNull(proxyHost);
    this.proxyPort = proxyPort;
    this.username = username;
    this.password = password;
    this.tls = tls;
  }

  @Override
  public Socket createSocket(final HttpContext context) {
    return new Socket(Proxy.NO_PROXY);
  }

  @Override
  public Socket connectSocket(
      final int connectTimeout,
      final Socket socket,
      final HttpHost host,
      final InetSocketAddress remoteAddress,
      @Nullable final InetSocketAddress localAddress,
      final HttpContext context) throws IOException
  {
    Socket sock = socket != null ? socket : createSocket(context);
    try {
      if (localAddress != null) {
        sock.bind(localAddress);
      }
      connectToProxy(sock, connectTimeout);
      int timeout = sock.getSoTimeout();
      if (connectTimeout > 0) {
        sock.setSoTimeout(connectTimeout);
      }
      try {
        handshake(sock, host.getHostName(), remoteAddress.getPort());
      }
      catch (SocketTimeoutException e) {
        throw new ProxyConnectException("The proxy did not answer the SOCKS5 handshake in time; is it a SOCKS5 proxy?",
            true);
      }
      sock.setSoTimeout(timeout);
    }
    catch (IOException | RuntimeException e) {
      closeQuietly(sock);
      throw e;
    }
    if (tls == null) {
      return sock;
    }
    return tls.createLayeredSocket(sock, host.getHostName(), remoteAddress.getPort(), context);
  }

  @Override
  public Socket createLayeredSocket(
      final Socket socket,
      final String target,
      final int port,
      final HttpContext context) throws IOException
  {
    if (tls == null) {
      throw new IOException("TLS is not available for " + target);
    }
    return tls.createLayeredSocket(socket, target, port, context);
  }

  private void connectToProxy(final Socket socket, final int connectTimeout) throws IOException {
    String address = proxyHost + ':' + proxyPort;
    InetSocketAddress proxy = new InetSocketAddress(proxyHost, proxyPort);
    if (proxy.isUnresolved()) {
      throw new ProxyConnectException("The proxy host " + proxyHost + " is unknown", true);
    }
    try {
      socket.connect(proxy, connectTimeout);
    }
    catch (SocketTimeoutException e) {
      throw new ProxyConnectException("The proxy did not answer in time: " + address, true);
    }
    catch (UnknownHostException e) {
      throw new ProxyConnectException("The proxy host " + proxyHost + " is unknown", true);
    }
    catch (IOException e) {
      throw new ProxyConnectException("The proxy refused the connection: " + address + " (" + e.getMessage() + ")",
          true);
    }
  }

  private void handshake(final Socket socket, final String host, final int port) throws IOException {
    OutputStream out = socket.getOutputStream();
    InputStream in = socket.getInputStream();
    boolean credentials = username != null;
    out.write(credentials
        ? new byte[]{VERSION, 2, NO_AUTHENTICATION, USERNAME_PASSWORD}
        : new byte[]{VERSION, 1, NO_AUTHENTICATION});
    out.flush();

    byte[] choice = read(in, 2);
    if (choice[0] != VERSION) {
      throw new ProxyConnectException("The proxy did not answer as a SOCKS5 proxy", false);
    }
    int method = choice[1] & 0xFF;
    if (method == USERNAME_PASSWORD && credentials) {
      authenticate(in, out);
    }
    else if (method == NO_ACCEPTABLE_METHOD) {
      throw new ProxyConnectException(credentials
          ? "The proxy accepts none of the offered authentication methods"
          : "The proxy requires a username and password", false);
    }
    else if (method != NO_AUTHENTICATION) {
      throw new ProxyConnectException("The proxy chose an authentication method that was not offered", false);
    }

    ByteArrayOutputStream request = new ByteArrayOutputStream();
    request.write(new byte[]{VERSION, CONNECT, 0});
    request.write(address(host));
    request.write(port >> 8);
    request.write(port);
    out.write(request.toByteArray());
    out.flush();

    byte[] reply = read(in, 4);
    if (reply[0] != VERSION) {
      throw new ProxyConnectException("The proxy did not answer as a SOCKS5 proxy", false);
    }
    int status = reply[1] & 0xFF;
    if (status != 0) {
      throw new ProxyConnectException("The proxy could not connect to " + host + ':' + port + ": " + reason(status),
          status != 2 && status != 7 && status != 8);
    }
    int length = switch (reply[3]) {
      case IPV4 -> 4;
      case IPV6 -> 16;
      case DOMAIN -> read(in, 1)[0] & 0xFF;
      default -> throw new ProxyConnectException("The proxy did not answer as a SOCKS5 proxy", false);
    };
    read(in, length + 2);
  }

  private void authenticate(final InputStream in, final OutputStream out) throws IOException {
    byte[] user = checkNotNull(username).getBytes(UTF_8);
    byte[] secret = password != null ? password.getBytes(UTF_8) : new byte[0];
    if (user.length > MAX_LENGTH || secret.length > MAX_LENGTH) {
      throw new ProxyConnectException("The proxy username or password is longer than SOCKS5 allows", false);
    }
    ByteArrayOutputStream request = new ByteArrayOutputStream();
    request.write(AUTHENTICATION_VERSION);
    request.write(user.length);
    request.write(user);
    request.write(secret.length);
    request.write(secret);
    out.write(request.toByteArray());
    out.flush();
    byte[] status = read(in, 2);
    if (status[1] != 0) {
      throw new ProxyConnectException("The proxy rejected the username or password", false);
    }
  }

  private static byte[] address(final String host) throws IOException {
    if (InetAddresses.isInetAddress(host)) {
      byte[] address = InetAddresses.forString(host).getAddress();
      byte[] encoded = new byte[address.length + 1];
      encoded[0] = (byte) (address.length == 4 ? IPV4 : IPV6);
      System.arraycopy(address, 0, encoded, 1, address.length);
      return encoded;
    }
    byte[] name = IDN.toASCII(host).getBytes(US_ASCII);
    if (name.length > MAX_LENGTH) {
      throw new ProxyConnectException("The host name " + host + " is longer than SOCKS5 allows", false);
    }
    byte[] encoded = new byte[name.length + 2];
    encoded[0] = DOMAIN;
    encoded[1] = (byte) name.length;
    System.arraycopy(name, 0, encoded, 2, name.length);
    return encoded;
  }

  private static byte[] read(final InputStream in, final int length) throws IOException {
    byte[] bytes = in.readNBytes(length);
    if (bytes.length < length) {
      throw new ProxyConnectException("The proxy closed the connection during the SOCKS5 handshake", true);
    }
    return bytes;
  }

  private static String reason(final int status) {
    return switch (status) {
      case 1 -> "general failure";
      case 2 -> "not allowed by the proxy rules";
      case 3 -> "network unreachable";
      case 4 -> "host unreachable";
      case 5 -> "connection refused";
      case 6 -> "TTL expired";
      case 7 -> "command not supported";
      case 8 -> "address type not supported";
      default -> "error " + status;
    };
  }

  private static void closeQuietly(final Socket socket) {
    try {
      socket.close();
    }
    catch (IOException e) {
      // nothing left to do with a socket that failed to connect
    }
  }
}
