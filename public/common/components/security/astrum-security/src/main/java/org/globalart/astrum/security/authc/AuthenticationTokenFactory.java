
package org.globalart.astrum.security.authc;

import javax.annotation.Nullable;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;

import org.apache.shiro.authc.AuthenticationToken;

/**
 * Factory of authentication tokens.
 *
 * @since 2.7
 */
public interface AuthenticationTokenFactory
{
  /**
   * Creates an {@link AuthenticationToken} based on give request/response, or if unable.
   */
  @Nullable
  AuthenticationToken createToken(ServletRequest request, ServletResponse response);
}
