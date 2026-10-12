package org.globalart.astrum.repository.terraform;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformFormat
    extends Format
{
  public static final String NAME = "terraform";

  public TerraformFormat() {
    super(NAME);
  }
}
