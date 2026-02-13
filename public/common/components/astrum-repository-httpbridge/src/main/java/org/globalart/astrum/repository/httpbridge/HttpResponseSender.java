
package org.globalart.astrum.repository.httpbridge;

import java.io.IOException;

import javax.annotation.Nullable;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

/**
 * Allows repository format specific handling of HTTP response sending.
 *
 * @since 3.0
 */
public interface HttpResponseSender
{
  void send(
      @Nullable Request request,
      Response response,
      HttpServletResponse httpServletResponse) throws ServletException, IOException;
}
