
package org.globalart.astrum.repository.apt.internal;

import javax.annotation.Nullable;

import org.globalart.astrum.mime.MimeRule;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.apt.AptFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(AptFormat.NAME)
public class AptMimeRulesSource
    implements MimeRulesSource
{
  private static final MimeRule SIGNATURE_RULE = new MimeRule(true, AptMimeTypes.SIGNATURE);

  @Nullable
  @Override
  public MimeRule getRuleForName(final String name) {
    // GPG signatures should always be served as application/pgp-signature
    if (name.endsWith(".gpg")) {
      return SIGNATURE_RULE;
    }

    return null;
  }
}
