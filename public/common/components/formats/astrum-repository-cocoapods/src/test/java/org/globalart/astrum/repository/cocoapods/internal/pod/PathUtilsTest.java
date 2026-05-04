
package org.globalart.astrum.repository.cocoapods.internal.pod;

import java.net.URI;

import org.globalart.astrum.repository.cocoapods.internal.PathUtils;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class PathUtilsTest
{
  @Test
  public void testGeneratePodPath() {
    String[][] cases = {
        {
            "mycheck888", "1.2.3", "https://github.com/mycheck888/MyCheckWalletUI/archive/1.2.3.tar.gz",
            "pods/mycheck888/1.2.3/1.2.3.tar.gz"
        },
        {
            "mycheck888", "1.2.3", "https://github.com/mycheck888/MyCheckWalletUI/archive/HEAD.tar.gz",
            "pods/mycheck888/1.2.3/HEAD.tar.gz"
        },
        {
            "packName", "0.9.2", "https://bitbucket.org/jefrydagucci/asbaseiosproject/get/v0.9.2.tar.gz",
            "pods/packName/0.9.2/v0.9.2.tar.gz"
        },
        {
            "packName", "0.9.2", "https://gitlab.com/streethawk/sdks/streethawk-sdk-ios/-/archive/1.10.2/1.10.2.tar.gz",
            "pods/packName/0.9.2/1.10.2.tar.gz"
        },
        {
            "AppSpectorTVSDK", "1.0.0", "https://github.com/appspector/ios-sdk/blob/master/AppSpectorTVSDK.zip?raw=true",
            "pods/AppSpectorTVSDK/1.0.0/AppSpectorTVSDK.zip"
        },
        {
            "packName", "0.9.2", "https://example.com/download/",
            "pods/packName/0.9.2/0.9.2"
        }
    };
    for (String[] testCase : cases) {
      assertThat(PathUtils.buildPodPath(testCase[0], testCase[1], URI.create(testCase[2])), is(testCase[3]));
    }
  }

  @Test
  public void testPodUriEscapesSegments() {
    URI uri = PathUtils.buildPodUri(URI.create("http://localhost/repository/cocoapods-proxy/"), "Pod", "1.0.0",
        URI.create("https://example.com/files/Pod%20SDK.zip"));
    assertThat(uri.toString(), is("http://localhost/repository/cocoapods-proxy/pods/Pod/1.0.0/Pod%20SDK.zip"));
  }

  @Test
  public void testSpecFilePath() {
    assertThat(PathUtils.buildSpecFilePath("Alamofire", "5.10.2"),
        is("Specs/d/a/2/Alamofire/5.10.2/Alamofire.podspec.json"));
  }

  @Test
  public void testValidSegment() {
    assertThat(PathUtils.isValidSegment("Alamofire"), is(true));
    assertThat(PathUtils.isValidSegment("1.0.0-beta.1"), is(true));
    assertThat(PathUtils.isValidSegment(".."), is(false));
    assertThat(PathUtils.isValidSegment("."), is(false));
    assertThat(PathUtils.isValidSegment(""), is(false));
    assertThat(PathUtils.isValidSegment("a/b"), is(false));
  }
}
