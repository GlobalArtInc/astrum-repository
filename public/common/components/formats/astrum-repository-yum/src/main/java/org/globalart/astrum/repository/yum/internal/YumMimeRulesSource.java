package org.globalart.astrum.repository.yum.internal;

import javax.annotation.Nullable;

import org.globalart.astrum.mime.MimeRule;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.yum.YumFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(YumFormat.NAME)
public class YumMimeRulesSource
    implements MimeRulesSource
{
  private static final MimeRule RPM_RULE = new MimeRule(false, "application/x-rpm");

  private static final MimeRule SIGNATURE_RULE = new MimeRule(true, "application/pgp-signature", "text/plain");

  private static final MimeRule KEY_RULE = new MimeRule(true, "application/pgp-keys", "text/plain");

  @Nullable
  @Override
  public MimeRule getRuleForName(final String name) {
    if (name.endsWith(YumPaths.RPM_EXTENSION)) {
      return RPM_RULE;
    }
    if (name.endsWith(".asc")) {
      return SIGNATURE_RULE;
    }
    if (name.endsWith(".key")) {
      return KEY_RULE;
    }
    return null;
  }
}
