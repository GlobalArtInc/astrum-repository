
package org.globalart.astrum.repository.p2.internal;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(P2Format.NAME)
public class P2Format
    extends Format
{
  public static final String NAME = "p2";

  public P2Format() {
    super(NAME);
  }
}
