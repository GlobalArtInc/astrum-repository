
package org.globalart.astrum.repository.httpbridge.internal.describe;

import java.net.URL;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.template.EscapeHelper;
import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default {@link DescriptionRenderer}.
 *
 * @since 3.0
 */
@Component
public class DescriptionRendererImpl
    implements DescriptionRenderer
{
  private static final String TEMPLATE_RESOURCE = "describeHtml.vm";

  private final TemplateHelper templateHelper;

  private final ObjectMapper objectMapper;

  private final URL template;

  @Autowired
  public DescriptionRendererImpl(final TemplateHelper templateHelper) {
    this.templateHelper = checkNotNull(templateHelper);
    objectMapper = new ObjectMapper();
    objectMapper.enable(SerializationFeature.INDENT_OUTPUT);
    template = getClass().getResource(TEMPLATE_RESOURCE);
    checkNotNull(template);
  }

  @Override
  public String renderHtml(final Description description) {
    TemplateParameters params = templateHelper.parameters();
    params.setAll(description.getParameters());
    params.set("items", description.getItems());
    params.set("esc", new EscapeHelper());
    return templateHelper.render(template, params);
  }

  @Override
  public String renderJson(final Description description) {
    try {
      return objectMapper.writeValueAsString(description);
    }
    catch (JsonProcessingException e) {
      throw new RuntimeException(e);
    }
  }
}
