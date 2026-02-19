
package org.globalart.astrum.repository.proxy;

public interface ThrottlerInterceptor
{
  String PAYMENT_REQUIRED_MESSAGE =
      "PAYMENT REQUIRED: At current usage levels, GlobalArt Astrum Repository Community Edition requires a paid license to"
          +
          " publish or cache new components. Contact your repository administrator to resolve. INSTANCE ID: ";

  boolean shouldBlock();
}
