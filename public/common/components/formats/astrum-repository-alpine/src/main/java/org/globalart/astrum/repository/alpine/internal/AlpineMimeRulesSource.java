package org.globalart.astrum.repository.alpine.internal;

import javax.annotation.Nullable;

import org.globalart.astrum.mime.MimeRule;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.alpine.AlpineFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(AlpineFormat.NAME)
public class AlpineMimeRulesSource
    implements MimeRulesSource
{
  private static final MimeRule PACKAGE_RULE = new MimeRule(true, "application/gzip", "application/x-gzip");

  @Nullable
  @Override
  public MimeRule getRuleForName(final String name) {
    if (name.endsWith(AlpinePathUtils.PACKAGE_EXTENSION)) {
      return PACKAGE_RULE;
    }
    return null;
  }
}
