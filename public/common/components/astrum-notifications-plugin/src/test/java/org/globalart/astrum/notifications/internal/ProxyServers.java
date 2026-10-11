package org.globalart.astrum.notifications.internal;

import java.io.BufferedInputStream;
import java.io.ByteArrayOutputStream;
import java.io.Closeable;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.URI;
import java.util.ArrayList;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicInteger;

import static java.nio.charset.StandardCharsets.ISO_8859_1;
import static java.nio.charset.StandardCharsets.US_ASCII;
import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Small proxies for tests that forward every host name to the loopback address, as if they resolved it themselves.
 */
final class ProxyServers
{
  private ProxyServers() {
  }

  abstract static class Server
      implements Closeable
  {
    private final ServerSocket server;

    private final ExecutorService threads = Executors.newCachedThreadPool();

    final AtomicInteger connections = new AtomicInteger();

    Server() throws IOException {
      server = new ServerSocket(0, 50, InetAddress.getLoopbackAddress());
      threads.execute(() -> {
        while (!server.isClosed()) {
          try {
            Socket client = server.accept();
            connections.incrementAndGet();
            threads.execute(() -> {
              try (client) {
                handle(client);
              }
              catch (IOException e) {
                // the client went away
              }
            });
          }
          catch (IOException e) {
            return;
          }
        }
      });
    }

    int port() {
      return server.getLocalPort();
    }

    abstract void handle(Socket client) throws IOException;

    void relay(final Socket client, final InputStream in, final int port) throws IOException {
      try (Socket target = new Socket(InetAddress.getLoopbackAddress(), port)) {
        threads.execute(() -> {
          try {
            in.transferTo(target.getOutputStream());
          }
          catch (IOException e) {
            // either side closed
          }
        });
        target.getInputStream().transferTo(client.getOutputStream());
      }
    }

    @Override
    public void close() throws IOException {
      server.close();
      threads.shutdownNow();
    }
  }

  /**
   * An HTTP proxy that forwards absolute-URI requests, answers CONNECT with {@link #connectStatus} and, when
   * {@link #credentials} are set, asks for Basic authentication.
   */
  static class HttpProxy
      extends Server
  {
    final List<String> requestLines = new CopyOnWriteArrayList<>();

    final List<String> authorizations = new CopyOnWriteArrayList<>();

    volatile String credentials;

    volatile String connectStatus = "403 Forbidden";

    HttpProxy() throws IOException {
    }

    @Override
    void handle(final Socket client) throws IOException {
      InputStream in = new BufferedInputStream(client.getInputStream());
      OutputStream out = client.getOutputStream();
      String requestLine = line(in);
      if (requestLine == null) {
        return;
      }
      Map<String, String> headers = new LinkedHashMap<>();
      List<String> forwarded = new ArrayList<>();
      for (String header = line(in); header != null && !header.isEmpty(); header = line(in)) {
        int colon = header.indexOf(':');
        String name = header.substring(0, colon).trim().toLowerCase();
        headers.put(name, header.substring(colon + 1).trim());
        if (!name.startsWith("proxy-") && !name.equals("connection")) {
          forwarded.add(header);
        }
      }
      byte[] body = in.readNBytes(Integer.parseInt(headers.getOrDefault("content-length", "0")));
      requestLines.add(requestLine);
      authorizations.add(headers.getOrDefault("proxy-authorization", ""));

      String expected = credentials == null ? null
          : "Basic " + Base64.getEncoder().encodeToString(credentials.getBytes(UTF_8));
      if (expected != null && !expected.equals(headers.get("proxy-authorization"))) {
        out.write(("HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"test\"\r\n"
            + "Content-Length: 0\r\nConnection: close\r\n\r\n").getBytes(US_ASCII));
        return;
      }
      String[] parts = requestLine.split(" ");
      if ("CONNECT".equals(parts[0])) {
        out.write(("HTTP/1.1 " + connectStatus + "\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .getBytes(US_ASCII));
        return;
      }
      URI uri = URI.create(parts[1]);
      try (Socket target = new Socket(InetAddress.getLoopbackAddress(), uri.getPort())) {
        StringBuilder head = new StringBuilder(parts[0]).append(' ').append(uri.getRawPath());
        if (uri.getRawQuery() != null) {
          head.append('?').append(uri.getRawQuery());
        }
        head.append(" HTTP/1.1\r\n");
        forwarded.forEach(header -> head.append(header).append("\r\n"));
        head.append("Connection: close\r\n\r\n");
        OutputStream targetOut = target.getOutputStream();
        targetOut.write(head.toString().getBytes(ISO_8859_1));
        targetOut.write(body);
        targetOut.flush();
        target.getInputStream().transferTo(out);
      }
    }

    private static String line(final InputStream in) throws IOException {
      ByteArrayOutputStream line = new ByteArrayOutputStream();
      for (int b = in.read(); b != -1; b = in.read()) {
        if (b == '\n') {
          return line.toString(ISO_8859_1).replaceAll("\r$", "");
        }
        line.write(b);
      }
      return line.size() > 0 ? line.toString(ISO_8859_1) : null;
    }
  }

  /**
   * A SOCKS5 proxy that records the address type and address of every request and, when {@link #username} is set,
   * asks for username and password authentication. A non-zero {@link #reply} refuses every request with that code.
   */
  static class Socks5Proxy
      extends Server
  {
    final List<String> requests = new CopyOnWriteArrayList<>();

    final List<String> logins = new CopyOnWriteArrayList<>();

    volatile String username;

    volatile String password;

    volatile int reply;

    Socks5Proxy() throws IOException {
    }

    @Override
    void handle(final Socket client) throws IOException {
      InputStream in = new BufferedInputStream(client.getInputStream());
      OutputStream out = client.getOutputStream();
      if (in.read() != 5) {
        return;
      }
      byte[] methods = in.readNBytes(in.read());
      if (username != null) {
        if (!contains(methods, 2)) {
          out.write(new byte[]{5, (byte) 0xFF});
          return;
        }
        out.write(new byte[]{5, 2});
        in.read();
        String user = new String(in.readNBytes(in.read()), UTF_8);
        String pass = new String(in.readNBytes(in.read()), UTF_8);
        logins.add(user + ":" + pass);
        boolean accepted = username.equals(user) && password.equals(pass);
        out.write(new byte[]{1, (byte) (accepted ? 0 : 1)});
        if (!accepted) {
          return;
        }
      }
      else {
        out.write(new byte[]{5, 0});
      }
      byte[] head = in.readNBytes(4);
      String address = switch (head[3]) {
        case 1 -> "ipv4 " + InetAddress.getByAddress(in.readNBytes(4)).getHostAddress();
        case 3 -> "domain " + new String(in.readNBytes(in.read()), US_ASCII);
        case 4 -> "ipv6 " + InetAddress.getByAddress(in.readNBytes(16)).getHostAddress();
        default -> "unknown";
      };
      int port = (in.read() << 8) | in.read();
      requests.add(address + ":" + port);
      if (reply != 0) {
        out.write(new byte[]{5, (byte) reply, 0, 1, 0, 0, 0, 0, 0, 0});
        return;
      }
      out.write(new byte[]{5, 0, 0, 3, 9, 'l', 'o', 'c', 'a', 'l', 'h', 'o', 's', 't', (byte) (port >> 8),
          (byte) port});
      out.flush();
      relay(client, in, port);
    }

    private static boolean contains(final byte[] values, final int value) {
      for (byte b : values) {
        if (b == value) {
          return true;
        }
      }
      return false;
    }
  }
}
