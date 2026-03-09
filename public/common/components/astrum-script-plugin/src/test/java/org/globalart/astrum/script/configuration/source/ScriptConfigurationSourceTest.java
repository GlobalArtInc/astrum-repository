
package org.globalart.astrum.script.configuration.source;

import java.util.List;

import org.globalart.astrum.configuration.model.InstanceConfigurationXO;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.script.ScriptManager;
import org.globalart.astrum.script.configuration.model.ScriptConfigurationListXO;
import org.globalart.astrum.script.configuration.model.ScriptConfigurationXO;

import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@SpringBootTest(classes = ScriptConfigurationSourceTest.TestConfiguration.class)
public class ScriptConfigurationSourceTest
{
  private final ScriptManager scriptManager;

  private final Script script;

  private final ArgumentCaptor<String> nameArgumentCaptor;

  private final ArgumentCaptor<String> typeArgumentCaptor;

  private final ArgumentCaptor<String> contentArgumentCaptor;

  private final ScriptConfigurationSource underTest;

  @Autowired
  public ScriptConfigurationSourceTest(
      final ScriptManager scriptManager,
      final Script script,
      @Qualifier("nameArgumentCaptor") final ArgumentCaptor<String> nameArgumentCaptor,
      @Qualifier("typeArgumentCaptor") final ArgumentCaptor<String> typeArgumentCaptor,
      @Qualifier("contentArgumentCaptor") final ArgumentCaptor<String> contentArgumentCaptor,
      final ScriptConfigurationSource underTest)
  {
    this.scriptManager = scriptManager;
    this.script = script;
    this.nameArgumentCaptor = nameArgumentCaptor;
    this.typeArgumentCaptor = typeArgumentCaptor;
    this.contentArgumentCaptor = contentArgumentCaptor;
    this.underTest = underTest;
  }

  @Test
  public void testApplyToExport() {
    // Arrange
    when(script.getName()).thenReturn("testScript");
    when(script.getType()).thenReturn("testType");
    when(script.getContent()).thenReturn("testContent");
    when(scriptManager.browse()).thenReturn(List.of(script));

    ScriptConfigurationXO expectedXo = new ScriptConfigurationXO();
    expectedXo.setName("testScript");
    expectedXo.setType("testType");
    expectedXo.setContent("testContent");

    // Act
    InstanceConfigurationXO exportXo = new InstanceConfigurationXO();
    underTest.applyToExport(exportXo);

    // Assert
    ScriptConfigurationListXO
        listXO =
        (ScriptConfigurationListXO) exportXo.getConfigurationXO(ScriptConfigurationListXO.TYPE_ID);
    assertThat(listXO, notNullValue());
    assertThat(listXO.getScriptConfigurationXOs(), hasSize(1));
    assertThat(listXO.getScriptConfigurationXOs().get(0), is(expectedXo));
  }

  @Test
  public void testApplyToInstance() {
    // Arrange
    ScriptConfigurationListXO listXO = new ScriptConfigurationListXO();
    ScriptConfigurationXO xo = new ScriptConfigurationXO();
    xo.setName("testScript");
    xo.setType("testType");
    xo.setContent("testContent");
    listXO.addScriptConfigurationXO(xo);

    // Act
    InstanceConfigurationXO configurationXO = new InstanceConfigurationXO();
    configurationXO.addConfigurationXO(listXO);
    underTest.applyToInstance(configurationXO);

    // Assert
    verify(scriptManager).create(
        nameArgumentCaptor.capture(),
        contentArgumentCaptor.capture(),
        typeArgumentCaptor.capture());
    assertThat(nameArgumentCaptor.getValue(), is("testScript"));
    assertThat(typeArgumentCaptor.getValue(), is("testType"));
    assertThat(contentArgumentCaptor.getValue(), is("testContent"));
  }

  @Configuration
  static class TestConfiguration
  {
    @Bean("nameArgumentCaptor")
    public ArgumentCaptor<String> nameArgumentCaptor() {
      return ArgumentCaptor.captor();
    }

    @Bean("typeArgumentCaptor")
    public ArgumentCaptor<String> typeArgumentCaptor() {
      return ArgumentCaptor.captor();
    }

    @Bean("contentArgumentCaptor")
    public ArgumentCaptor<String> contentArgumentCaptor() {
      return ArgumentCaptor.captor();
    }

    @Bean
    public Script script() {
      return mock();
    }

    @Bean
    public ScriptManager scriptManager() {
      return mock();
    }

    @Bean
    public ScriptConfigurationSource scriptConfigurationSource() {
      return new ScriptConfigurationSource(scriptManager());
    }
  }
}
