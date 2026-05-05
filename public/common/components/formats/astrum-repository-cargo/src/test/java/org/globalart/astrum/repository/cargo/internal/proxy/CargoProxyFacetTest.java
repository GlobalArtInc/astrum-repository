package org.globalart.astrum.repository.cargo.internal.proxy;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class CargoProxyFacetTest
{
  @Test
  public void prefixes() {
    assertThat(CargoProxyFacet.prefix("a"), is(equalTo("1")));
    assertThat(CargoProxyFacet.prefix("ab"), is(equalTo("2")));
    assertThat(CargoProxyFacet.prefix("Abc"), is(equalTo("3/A")));
    assertThat(CargoProxyFacet.prefix("Cargo"), is(equalTo("Ca/rg")));
  }

  @Test
  public void detectsTemplateMarkers() {
    assertThat(CargoProxyFacet.hasMarkers("https://static.crates.io/crates"), is(false));
    assertThat(CargoProxyFacet.hasMarkers("https://example.com/{crate}/{version}.crate"), is(true));
    assertThat(CargoProxyFacet.hasMarkers("https://example.com/{lowerprefix}/{crate}"), is(true));
    assertThat(CargoProxyFacet.hasMarkers("https://example.com/{sha256-checksum}"), is(true));
  }
}
