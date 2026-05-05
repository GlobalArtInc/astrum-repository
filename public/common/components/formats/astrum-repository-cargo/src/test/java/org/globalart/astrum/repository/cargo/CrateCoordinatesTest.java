package org.globalart.astrum.repository.cargo;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class CrateCoordinatesTest
{
  @Test
  public void indexPaths() {
    assertThat(CrateCoordinates.indexPath("a"), is(equalTo("1/a")));
    assertThat(CrateCoordinates.indexPath("ab"), is(equalTo("2/ab")));
    assertThat(CrateCoordinates.indexPath("abc"), is(equalTo("3/a/abc")));
    assertThat(CrateCoordinates.indexPath("cargo"), is(equalTo("ca/rg/cargo")));
    assertThat(CrateCoordinates.indexPath("Serde_JSON"), is(equalTo("se/rd/serde_json")));
  }

  @Test
  public void namesFromIndexPaths() {
    assertThat(CrateCoordinates.nameFromIndexPath("/1/a"), is(equalTo("a")));
    assertThat(CrateCoordinates.nameFromIndexPath("/2/ab"), is(equalTo("ab")));
    assertThat(CrateCoordinates.nameFromIndexPath("/3/s/syn"), is(equalTo("syn")));
    assertThat(CrateCoordinates.nameFromIndexPath("/se/rd/serde"), is(equalTo("serde")));
    assertThat(CrateCoordinates.nameFromIndexPath("/se/rd/Serde"), is(equalTo("serde")));
    assertThat(CrateCoordinates.nameFromIndexPath("/se/rx/serde"), is(nullValue()));
    assertThat(CrateCoordinates.nameFromIndexPath("/3/x/syn"), is(nullValue()));
    assertThat(CrateCoordinates.nameFromIndexPath("/config.json"), is(nullValue()));
    assertThat(CrateCoordinates.nameFromIndexPath("/api/v1/crates"), is(nullValue()));
    assertThat(CrateCoordinates.nameFromIndexPath("/me"), is(nullValue()));
  }

  @Test
  public void validation() {
    assertThat(CrateCoordinates.isValidName("hello_world-2"), is(true));
    assertThat(CrateCoordinates.isValidName("9lives"), is(false));
    assertThat(CrateCoordinates.isValidName("a.b"), is(false));
    assertThat(CrateCoordinates.isValidName(""), is(false));
    assertThat(CrateCoordinates.isValidVersion("1.2.3"), is(true));
    assertThat(CrateCoordinates.isValidVersion("1.0.0-alpha.1+build.5"), is(true));
    assertThat(CrateCoordinates.isValidVersion("1.0"), is(false));
    assertThat(CrateCoordinates.isValidVersion("01.0.0"), is(false));
  }

  @Test
  public void assetPath() {
    CrateCoordinates coordinates = new CrateCoordinates("Hello", "0.1.0");
    assertThat(coordinates.getAssetPath(), is(equalTo("/crates/Hello/0.1.0/Hello-0.1.0.crate")));
    assertThat(coordinates.getLowerName(), is(equalTo("hello")));
    assertThat(coordinates.getIndexEntryPath(), is(equalTo("he/ll/hello")));
  }

  @Test(expected = IllegalArgumentException.class)
  public void rejectsInvalidCoordinates() {
    new CrateCoordinates("../etc", "1.0.0");
  }
}
