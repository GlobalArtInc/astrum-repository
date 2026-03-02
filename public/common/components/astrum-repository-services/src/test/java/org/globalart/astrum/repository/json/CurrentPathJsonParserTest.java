
package org.globalart.astrum.repository.json;

import java.io.IOException;

import com.fasterxml.jackson.core.JsonFactory;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.notNullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class CurrentPathJsonParserTest
{
  private final static String SIMPLE_JSON = "{\"_id\":\"simple\",\"user\":{\"description\":\"simplestuff\"}}";

  private CurrentPathJsonParser underTest;

  @Before
  public void setUp() throws IOException {
    underTest = new CurrentPathJsonParser(new JsonFactory().createParser(SIMPLE_JSON));
  }

  @Test
  public void should_Return_CurrentPath_For_Parser() throws IOException {
    assertThat(underTest.currentPath(), equalTo("/"));

    underTest.nextValue();
    assertThat(underTest.currentPath(), equalTo("/"));

    underTest.nextValue();
    assertThat(underTest.currentPath(), equalTo("/_id"));

    underTest.nextValue();
    assertThat(underTest.currentPath(), equalTo("/user"));

    underTest.nextValue();
    assertThat(underTest.currentPath(), equalTo("/user/description"));

    underTest.nextValue();
    assertThat(underTest.currentPath(), equalTo("/user"));

    underTest.nextValue();
    assertThat(underTest.currentPath(), equalTo("/"));
  }

  @Test
  public void should_Return_CurrentPath_InParts_For_Parser() throws IOException {
    assertThat(underTest.currentPathInParts().length, equalTo(0));

    underTest.nextValue();
    assertThat(underTest.currentPathInParts().length, equalTo(0));

    underTest.nextValue();
    assertThat(underTest.currentPathInParts().length, equalTo(1));
    assertThat(underTest.currentPathInParts()[0], equalTo("_id"));

    underTest.nextValue();
    assertThat(underTest.currentPathInParts().length, equalTo(1));
    assertThat(underTest.currentPathInParts()[0], equalTo("user"));

    underTest.nextValue();
    assertThat(underTest.currentPathInParts().length, equalTo(2));
    assertThat(underTest.currentPathInParts()[0], equalTo("user"));
    assertThat(underTest.currentPathInParts()[1], equalTo("description"));

    underTest.nextValue();
    assertThat(underTest.currentPathInParts().length, equalTo(1));
    assertThat(underTest.currentPathInParts()[0], equalTo("user"));

    underTest.nextValue();
    assertThat(underTest.currentPathInParts().length, equalTo(0));
  }

  @Test
  public void should_Return_CurrentPointer() {
    assertThat(underTest.currentPointer(), notNullValue());
  }
}
