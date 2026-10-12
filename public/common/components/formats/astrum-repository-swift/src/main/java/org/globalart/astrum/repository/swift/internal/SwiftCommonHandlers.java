package org.globalart.astrum.repository.swift.internal;

import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.security.anonymous.AnonymousHelper;

import com.google.common.net.HttpHeaders;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.springframework.stereotype.Component;

@Component
public class SwiftCommonHandlers
{
  public final Handler login = context -> {
    if (isAnonymous()) {
      Response response = SwiftResponses.problem(401, "sign in with the user name and password of a user");
      response.getHeaders().set(HttpHeaders.WWW_AUTHENTICATE, "Basic realm=\"Astrum Repository\"");
      return response;
    }
    return SwiftResponses.empty(200);
  };

  public final Handler availability = context -> SwiftResponses.json(SwiftResponses.object());

  public final Handler notFound = context -> SwiftResponses.problem(404, "not found");

  static boolean isAnonymous() {
    Subject subject = SecurityUtils.getSubject();
    return subject == null || subject.getPrincipal() == null || AnonymousHelper.isAnonymous(subject);
  }
}
