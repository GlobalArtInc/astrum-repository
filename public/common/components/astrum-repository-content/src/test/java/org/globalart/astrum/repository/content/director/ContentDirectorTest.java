
package org.globalart.astrum.repository.content.director;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponentBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponents;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
public class ContentDirectorTest
{
  @Mock
  private Repository source;

  @Mock
  private Repository destination;

  @Mock
  private Component component;

  @Mock
  private FluentComponent fluentComponent;

  private ContentDirector underTest;

  @BeforeEach
  public void setUp() {
    underTest = new ContentDirector()
    {
      // Use an anonymous implementation that just uses defaults
    };
  }

  @Test
  public void testBeforeMoveReturnsComponent() {
    List<Asset> assets = new ArrayList<>();
    Component result = underTest.beforeMove(component, assets, source, destination);
    assertThat(result, is(component));
  }

  @Test
  public void testAfterMoveReturnsComponent() {
    Component result = underTest.afterMove(component, destination);
    assertThat(result, is(component));
  }

  @Test
  public void testAllowMoveToReturnsFalse() {
    assertThat(underTest.allowMoveTo(destination), is(false));
  }

  @Test
  public void testAllowMoveToComponentReturnsFalse() {
    assertThat(underTest.allowMoveTo(fluentComponent, destination), is(false));
  }

  @Test
  public void testAllowMoveFromReturnsFalse() {
    assertThat(underTest.allowMoveFrom(source), is(false));
  }

  @Test
  public void testAfterMoveListIsNoOp() {
    // Should not throw
    underTest.afterMove(Collections.emptyList(), destination);
  }

  @Test
  public void testRedeployAllowedWhenWritePolicyAllow() {
    Configuration config = mock(Configuration.class);
    NestedAttributesMap attributes = mock();

    when(destination.getConfiguration()).thenReturn(config);
    when(config.attributes("storage")).thenReturn(attributes);
    when(attributes.get("writePolicy")).thenReturn(WritePolicy.ALLOW.name());

    assertThat(underTest.redeployAllowed(destination, component), is(true));
  }

  @Test
  public void testRedeployNotAllowedWhenWritePolicyDeny() {
    Configuration config = mock(Configuration.class);
    NestedAttributesMap attributes = mock();

    when(destination.getConfiguration()).thenReturn(config);
    when(config.attributes("storage")).thenReturn(attributes);
    when(attributes.get("writePolicy")).thenReturn("DENY");

    assertThat(underTest.redeployAllowed(destination, component), is(false));
  }

  @Test
  public void testCopyComponentDelegatesToContentFacet() {
    ContentFacet contentFacet = mock(ContentFacet.class);
    FluentComponents components = mock(FluentComponents.class);
    FluentComponentBuilder builder = mock(Answers.RETURNS_SELF);
    FluentComponent created = mock(FluentComponent.class);

    when(destination.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(contentFacet.components()).thenReturn(components);
    when(components.name(anyString())).thenReturn(builder);
    when(builder.getOrCreate()).thenReturn(created);

    when(component.name()).thenReturn("artifact");
    when(component.namespace()).thenReturn("org.example");
    when(component.version()).thenReturn("1.0");
    when(component.attributes()).thenReturn(new NestedAttributesMap("attributes", new HashMap<>()));
    when(component.kind()).thenReturn("provider");

    FluentComponent result = underTest.copyComponent(component, destination);
    assertThat(result, is(created));

    verify(components).name("artifact");
    verify(builder).namespace("org.example");
    verify(builder).version("1.0");
    verify(builder).kind("provider");
    verify(builder).getOrCreate();
  }
}
