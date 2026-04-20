
package org.globalart.astrum.repository.composer.internal.hosted;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import com.google.common.collect.ImmutableMap;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerContentFacet;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.ComposerHostedFacet;
import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.composer.internal.ComposerPathUtils;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import java.io.IOException;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default implementation of a Composer hosted facet.
 */
@Component
@Qualifier(ComposerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ComposerHostedFacetImpl
    extends FacetSupport
    implements ComposerHostedFacet
{
  private static final Pattern FILTER_PATTERN = Pattern.compile("\\s*(?<vendor>[*a-zA-Z0-9_.-]+)/(?<project>[*a-zA-Z0-9_.-]+)\\s*");

  private final ComposerJsonProcessor composerJsonProcessor;

  @Autowired
  public ComposerHostedFacetImpl(final ComposerJsonProcessor composerJsonProcessor) {
    this.composerJsonProcessor = checkNotNull(composerJsonProcessor);
  }

  @Override
  public FluentAsset upload(final String vendor, final String project, final String version, final String sourceType,
                            final String sourceUrl, final String sourceReference, final Payload payload)
      throws IOException
  {
    return content().put(
        ComposerPathUtils.buildZipballPath(vendor, project, version),
        payload,
        sourceType,
        sourceUrl,
        sourceReference
    );
  }

  @Override
  public Content getZipball(final String path) throws IOException {
    return content().get(path).orElse(null);
  }

  @Override
  public Content getPackagesJson() throws IOException {
    return composerJsonProcessor.generatePackagesFromComponents(getRepository(), content().components());
  }

  @Override
  public Content getListJson(String filter) throws IOException {
    FluentQuery<FluentComponent> components;
    if (filter == null || filter.isEmpty()) {
      components = content().components();
    } else {
      components = queryComponents(filter);
    }

    return composerJsonProcessor.generateListFromComponents(components);
  }

  @Override
  public Content getProviderJson(final String vendor, final String project) throws IOException {
    Optional<Content> content = content().get(ComposerPathUtils.buildProviderPath(vendor, project));
    if (content.isPresent()) {
      return content.get();
    } else {
      return rebuildProviderJson(vendor, project).orElse(null);
    }
  }

  @Override
  public Content getPackageJson(final String vendor, final String project) throws IOException {
    Optional<Content> content = content().get(ComposerPathUtils.buildPackagePath(vendor, project));
    //Create v2 Package if it´s not existing
    if (content.isPresent()) {
      return content.get();
    } else {
      return rebuildPackageJson(vendor, project).orElse(null);
    }
  }

  @Override
  public Optional<Content> rebuildProviderJson(final String vendor, final String project) throws IOException {
    Optional<Content> content = composerJsonProcessor.buildProviderJson(getRepository(), content(), queryComponents(vendor, project));
    if (content.isPresent()) {
      content().put(ComposerPathUtils.buildProviderPath(vendor, project), content.get(), AssetKind.PROVIDER);
    } else {
      content()
          .getAsset(ComposerPathUtils.buildProviderPath(vendor, project))
          .ifPresent(FluentAsset::delete);
    }
    return content;
  }

  @Override
  public Optional<Content>  rebuildPackageJson(final String vendor, final String project) throws IOException {
    Optional<Content>  content = composerJsonProcessor.buildPackageJson(getRepository(), content(), queryComponents(vendor, project));
    if (content.isPresent()) {
      content().put(ComposerPathUtils.buildPackagePath(vendor, project), content.get(), AssetKind.PACKAGE);
    } else {
      content()
          .getAsset(ComposerPathUtils.buildPackagePath(vendor, project))
          .ifPresent(FluentAsset::delete);
    }
    return content;
  }

  private FluentQuery<FluentComponent> queryComponents(final String vendor, final String project) {
    return content()
        .components()
        .byFilter(
            "namespace = #{filterParams.vendor} AND name = #{filterParams.project}",
            ImmutableMap.of("vendor", vendor, "project", project)
        );
  }

  private FluentQuery<FluentComponent> queryComponents(final String filter) {
    Matcher m = FILTER_PATTERN.matcher(filter);
    if (m.matches()) {
      String vendor = m.group("vendor").replaceAll("\\*+", "%");
      String project = m.group("project").replaceAll("\\*+", "%");

      return content()
          .components()
          .byFilter(
              "namespace LIKE #{filterParams.vendor} AND name LIKE #{filterParams.project}",
              ImmutableMap.of("vendor", vendor, "project", project)
          );
    } else {
      // invalid filter pattern
      return null;
    }
  }

  private ComposerContentFacet content() {
    return getRepository().facet(ComposerContentFacet.class);
  }
}
