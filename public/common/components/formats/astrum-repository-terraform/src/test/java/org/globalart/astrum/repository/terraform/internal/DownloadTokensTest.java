package org.globalart.astrum.repository.terraform.internal;

import java.time.Clock;
import java.time.Instant;
import java.time.ZoneOffset;

import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;

public class DownloadTokensTest
{
  private static final String PATH = "/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_SHA256SUMS";

  private final Clock now = Clock.fixed(Instant.parse("2026-10-12T10:00:00Z"), ZoneOffset.UTC);

  private final DownloadTokens tokens = new DownloadTokens("secret".getBytes(UTF_8), now);

  @Test
  public void signsOnePathForALimitedTime() {
    String url = tokens.sign("../../terraform-provider-demo_1.0.0_SHA256SUMS", PATH);
    String expires = parameter(url, DownloadTokens.EXPIRES);
    String signature = parameter(url, DownloadTokens.SIGNATURE);

    assertThat(url).startsWith("../../terraform-provider-demo_1.0.0_SHA256SUMS?expires=");
    assertThat(tokens.isValid(PATH, expires, signature)).isTrue();
    assertThat(tokens.isValid(PATH + ".sig", expires, signature)).isFalse();
    assertThat(tokens.isValid(PATH, String.valueOf(Long.parseLong(expires) + 1), signature)).isFalse();
    assertThat(new DownloadTokens("other".getBytes(UTF_8), now).isValid(PATH, expires, signature)).isFalse();

    Clock later = Clock.offset(now, DownloadTokens.LIFETIME.plusSeconds(1));
    assertThat(new DownloadTokens("secret".getBytes(UTF_8), later).isValid(PATH, expires, signature)).isFalse();
    assertThat(tokens.isValid(PATH, null, signature)).isFalse();
    assertThat(tokens.isValid(PATH, "soon", signature)).isFalse();
  }

  @Test
  public void keepsTheSubdirectoryOfAModuleLocation() {
    String location = tokens.sign("./vpc-aws-1.0.0.tar.gz//vpc", "/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz");

    assertThat(location).startsWith("./vpc-aws-1.0.0.tar.gz//vpc?expires=");
  }

  @Test
  public void signsAgainWhatAMemberSigned() {
    DownloadTokens member = new DownloadTokens("member".getBytes(UTF_8), now);
    String signed = member.sign("./vpc-aws-1.0.0.tar.gz//vpc", "/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz");

    String resigned = tokens.resign(signed, "/v1/modules/acme/vpc/aws/1.0.0/download");

    assertThat(resigned).startsWith("./vpc-aws-1.0.0.tar.gz//vpc?expires=");
    assertThat(tokens.isValid("/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz",
        parameter(resigned, DownloadTokens.EXPIRES), parameter(resigned, DownloadTokens.SIGNATURE))).isTrue();
    assertThat(tokens.resign("git::https://github.com/acme/vpc?ref=v1", "/v1/modules/acme/vpc/aws/1.0.0/download"))
        .isEqualTo("git::https://github.com/acme/vpc?ref=v1");
    assertThat(tokens.resign("https://releases.example.com/a.zip?signature=x", "/v1/x"))
        .isEqualTo("https://releases.example.com/a.zip?signature=x");
  }

  @Test
  public void findsThePathARelativeUrlNames() {
    assertThat(DownloadTokens.targetPath("/v1/providers/acme/demo/1.0.0/download/linux/amd64",
        "../../terraform-provider-demo_1.0.0_linux_amd64.zip"))
        .isEqualTo("/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_linux_amd64.zip");
    assertThat(DownloadTokens.targetPath("/registry.terraform.io/hashicorp/null/3.2.3.json",
        "../../../v1/providers/hashicorp/null/3.2.3/a.zip")).isEqualTo("/v1/providers/hashicorp/null/3.2.3/a.zip");
  }

  private static String parameter(final String url, final String name) {
    for (String pair : url.substring(url.indexOf('?') + 1).split("&")) {
      if (pair.startsWith(name + "=")) {
        return pair.substring(name.length() + 1);
      }
    }
    return null;
  }
}
