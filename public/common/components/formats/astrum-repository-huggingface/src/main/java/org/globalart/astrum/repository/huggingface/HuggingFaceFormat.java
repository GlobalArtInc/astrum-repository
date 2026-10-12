package org.globalart.astrum.repository.huggingface;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceFormat
    extends Format
{
  public static final String NAME = "huggingface";

  public HuggingFaceFormat() {
    super(NAME);
  }
}
