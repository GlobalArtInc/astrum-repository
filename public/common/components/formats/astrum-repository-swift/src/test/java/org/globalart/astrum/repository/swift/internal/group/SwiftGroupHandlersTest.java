package org.globalart.astrum.repository.swift.internal.group;

import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class SwiftGroupHandlersTest
{
  @Test
  void rewritesMemberUrlsToTheGroup() {
    Response response = new Response.Builder()
        .status(Status.success(303))
        .header("Location", "https://h/repository/swift-hosted/mona/x/1.0.0/Package.swift")
        .header("Link", "<https://h/repository/swift-hosted/mona/x/1.0.0>; rel=\"latest-version\", "
            + "<https://github.com/mona/x>; rel=\"canonical\"")
        .build();
    SwiftGroupHandlers.rewrite(response, "https://h/repository/swift-group", "https://h/repository/swift-hosted");
    assertThat(response.getHeaders().get("Location"), is("https://h/repository/swift-group/mona/x/1.0.0/Package.swift"));
    assertThat(response.getHeaders().get("Link"), is("<https://h/repository/swift-group/mona/x/1.0.0>; "
        + "rel=\"latest-version\", <https://github.com/mona/x>; rel=\"canonical\""));
    assertThat(SwiftGroupHandlers.rewrite("https://h/repository/swift-hosted-2/a", "https://h/repository/g",
        "https://h/repository/swift-hosted"), is("https://h/repository/swift-hosted-2/a"));
  }

  @Test
  void readsPackageCaseFromReleaseUrls() {
    assertThat(SwiftGroupHandlers.fromReleaseUrl("https://h/repository/swift-proxy/acme/Toolkit/1.0.0"),
        is(new SwiftCoordinates("acme", "Toolkit", null)));
    assertThat(SwiftGroupHandlers.fromReleaseUrl("1.0.0"), is(nullValue()));
    assertThat(SwiftGroupHandlers.fromReleaseUrl(null), is(nullValue()));
  }
}
