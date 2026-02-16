
package org.globalart.astrum.repository.httpbridge.internal;

import java.util.Enumeration;

import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.repository.view.Parameters;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * HTTP {@link Parameters} adapter.
 *
 * @since 3.0
 */
class HttpParametersAdapter
    extends Parameters
{
  public HttpParametersAdapter(final HttpServletRequest request) {
    checkNotNull(request);
    Enumeration<String> names = request.getParameterNames();
    while (names.hasMoreElements()) {
      String name = names.nextElement();
      set(name, request.getParameterValues(name));
    }
  }
}
