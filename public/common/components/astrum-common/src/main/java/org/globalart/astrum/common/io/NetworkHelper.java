
package org.globalart.astrum.common.io;

import java.net.InetAddress;
import java.net.NetworkInterface;
import java.util.Enumeration;

/**
 * Offers static helper methods related to network.
 *
 * @since 3.7.0
 */
public final class NetworkHelper
{
  private NetworkHelper() {
  }

  /**
   * Workaround ambiguous InetAddress.getLocalHost() on Linux-based systems.
   *
   * @see <a href=
   *      "http://bugs.java.com/bugdatabase/view_bug.do?bug_id=4665037">http://bugs.java.com/bugdatabase/view_bug.do?bug_id=4665037</a>
   */
  public static String findLocalHostAddress() throws Exception {
    // first look for a local address which isn't the loopback interface or P2P/VPN
    Enumeration<NetworkInterface> interfaces = NetworkInterface.getNetworkInterfaces();
    while (interfaces.hasMoreElements()) {
      NetworkInterface intf = interfaces.nextElement();
      if (intf.isUp() && !intf.isLoopback() && !intf.isPointToPoint() && !intf.isVirtual()) {
        Enumeration<InetAddress> addresses = intf.getInetAddresses();
        while (addresses.hasMoreElements()) {
          InetAddress addr = addresses.nextElement();
          if (addr.isSiteLocalAddress() && !addr.isLoopbackAddress()) {
            return addr.getHostAddress();
          }
        }
      }
    }
    // otherwise fall back to the JDKs 'guesstimate'
    return InetAddress.getLocalHost().getHostAddress();
  }
}
