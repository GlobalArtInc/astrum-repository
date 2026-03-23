
package org.globalart.astrum.repository.maven.internal;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;

import org.springframework.stereotype.Component;

/**
 * @since 3.8
 */
@Component
public class MavenPomGenerator
{
  private final TemplateHelper templateHelper;

  @Autowired
  public MavenPomGenerator(final TemplateHelper templateHelper) {
    this.templateHelper = templateHelper;
  }

  public String generatePom(
      final String groupId,
      final String artifactId,
      final String version,
      @Nullable final String packaging)
  {
    TemplateParameters params = templateHelper.parameters();
    params.set("groupId", groupId);
    params.set("artifactId", artifactId);
    params.set("version", version);
    params.set("packaging", packaging);
    return templateHelper.render(getClass().getResource("pom.vm"), params);
  }
}
