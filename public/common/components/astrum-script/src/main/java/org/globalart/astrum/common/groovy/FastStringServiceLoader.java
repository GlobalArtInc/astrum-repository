
package org.globalart.astrum.common.groovy;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.common.thread.TcclBlock;

import org.apache.groovy.json.FastStringService;
import org.apache.groovy.json.internal.FastStringUtils;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import org.springframework.stereotype.Component;

/**
 * Sets the FastStringService context-class-loader to the current thread class-loader
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class FastStringServiceLoader
    extends StateGuardLifecycleSupport
{
  @Override
  protected void doStart() throws Exception {
    // HACK: workaround to 'Unable to load FastStringService' groovy issue
    try (TcclBlock ignored = TcclBlock.begin(FastStringService.class)) {
      FastStringUtils.toCharArray("ab");
    }
  }
}
