
package org.globalart.astrum.repository.content.store;

import java.util.Optional;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class ContentStoreEventTest
{
  @Test
  public void testToStringWithoutRepositorySupplier() {
    ContentStoreEvent event = new ContentStoreEvent(42)
    {
    };
    assertThat(event.toString(), containsString("contentRepositoryId=42"));
    assertThat(event.toString(), containsString("repository=null"));
  }

  @Test
  public void testToStringWithRepositorySupplier() {
    ContentStoreEvent event = new ContentStoreEvent(10)
    {
    };
    Repository repository = mock(Repository.class);
    when(repository.toString()).thenReturn("TestRepo");
    event.setRepositorySupplier(() -> Optional.of(repository));

    assertThat(event.toString(), containsString("contentRepositoryId=10"));
  }

  @Test
  public void testGetRepositoryWithSupplier() {
    ContentStoreEvent event = new ContentStoreEvent(1)
    {
    };
    Repository repository = mock(Repository.class);
    event.setRepositorySupplier(() -> Optional.of(repository));

    Optional<Repository> result = event.getRepository();
    assertThat(result.isPresent(), is(true));
    assertThat(result.get(), is(repository));
  }

  @Test
  public void testGetRepositoryWithEmptySupplier() {
    ContentStoreEvent event = new ContentStoreEvent(1)
    {
    };
    event.setRepositorySupplier(Optional::empty);

    Optional<Repository> result = event.getRepository();
    assertThat(result.isPresent(), is(false));
  }

  @Test(expected = IllegalStateException.class)
  public void testGetRepositoryWithoutSupplierThrows() {
    ContentStoreEvent event = new ContentStoreEvent(1)
    {
    };
    event.getRepository();
  }

  @Test(expected = IllegalStateException.class)
  public void testSetRepositorySupplierTwiceThrows() {
    ContentStoreEvent event = new ContentStoreEvent(1)
    {
    };
    event.setRepositorySupplier(Optional::empty);
    event.setRepositorySupplier(Optional::empty);
  }

  @Test
  public void testGetFormatReturnsFormatValue() {
    ContentStoreEvent event = new ContentStoreEvent(1)
    {
    };
    Repository repository = mock(Repository.class);
    Format format = new Format("maven2")
    {
    };
    when(repository.getFormat()).thenReturn(format);
    event.setRepositorySupplier(() -> Optional.of(repository));

    assertThat(event.getFormat(), is("maven2"));
  }

  @Test
  public void testGetFormatReturnsNullWhenNoRepository() {
    ContentStoreEvent event = new ContentStoreEvent(1)
    {
    };
    event.setRepositorySupplier(Optional::empty);

    assertThat(event.getFormat(), is(nullValue()));
  }
}
