
package org.globalart.astrum.repository.cocoapods.internal.git;

import java.net.URI;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class GitApiHelperTest
{
  private final GitApiHelper gitApiHelper =
      new GitApiHelper("https://github.com", "https://bitbucket.org", "https://gitlab.com/");

  @Test
  public void testGenerateDownloadUri() {
    String[][] cases = {
        {
            "https://github.com/mycheck888/MyCheckWalletUI.git", "1.2.3",
            "https://github.com/mycheck888/MyCheckWalletUI/archive/1.2.3.tar.gz"
        },
        {
            "https://github.com/mycheck888/MyCheckWalletUI.git", null,
            "https://github.com/mycheck888/MyCheckWalletUI/archive/HEAD.tar.gz"
        },
        {
            "https://github.com/Alamofire/Alamofire", "5.10.2",
            "https://github.com/Alamofire/Alamofire/archive/5.10.2.tar.gz"
        },
        {
            "https://bitbucket.org/jefrydagucci/asbaseiosproject.git", "v0.9.2",
            "https://bitbucket.org/jefrydagucci/asbaseiosproject/get/v0.9.2.tar.gz"
        },
        {
            "https://bitbucket.org/jefrydagucci/asbaseiosproject.git", null,
            "https://bitbucket.org/jefrydagucci/asbaseiosproject/get/master.tar.gz"
        },
        {
            "https://gitlab.com/streethawk/sdks/streethawk-sdk-ios.git", "1.10.2",
            "https://gitlab.com/streethawk/sdks/streethawk-sdk-ios/-/archive/1.10.2/1.10.2.tar.gz"
        },
        {
            "https://gitlab.com/streethawk/sdks/streethawk-sdk-ios.git", null,
            "https://gitlab.com/streethawk/sdks/streethawk-sdk-ios/-/archive/master/master.tar.gz"
        }
    };
    for (String[] testCase : cases) {
      assertThat(gitApiHelper.buildDownloadURI(URI.create(testCase[0]), testCase[1]).toString(), is(testCase[2]));
    }
  }

  @Test(expected = IllegalArgumentException.class)
  public void testUnsupportedHost() {
    gitApiHelper.buildDownloadURI(URI.create("https://git.example.com/vendor/repo.git"), "1.0");
  }
}
