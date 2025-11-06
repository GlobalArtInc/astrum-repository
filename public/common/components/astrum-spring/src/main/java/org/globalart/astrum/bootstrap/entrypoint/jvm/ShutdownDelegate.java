
package org.globalart.astrum.bootstrap.entrypoint.jvm;

public interface ShutdownDelegate
{
  void exit(int code);

  void halt(int code);
}
