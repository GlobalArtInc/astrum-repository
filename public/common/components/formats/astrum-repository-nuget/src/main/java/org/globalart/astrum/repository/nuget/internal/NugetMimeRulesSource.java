package org.globalart.astrum.repository.nuget.internal;

import java.util.Locale;

import javax.annotation.Nullable;

import org.globalart.astrum.mime.MimeRule;
import org.globalart.astrum.mime.MimeRulesSource;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetMimeRulesSource
    implements MimeRulesSource
{
  private static final MimeRule PACKAGE_RULE =
      new MimeRule(true, "application/zip", "application/x-tika-ooxml", "application/x-tika-msoffice",
          "application/octet-stream");

  @Nullable
  @Override
  public MimeRule getRuleForName(final String name) {
    String lower = name.toLowerCase(Locale.ROOT);
    if (lower.endsWith(".nupkg") || lower.endsWith(".snupkg")) {
      return PACKAGE_RULE;
    }
    return null;
  }
}
