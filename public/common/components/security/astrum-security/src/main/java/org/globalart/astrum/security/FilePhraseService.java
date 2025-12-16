
package org.globalart.astrum.security;

import java.io.File;
import java.io.IOException;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.AbstractPhraseService;
import org.globalart.astrum.crypto.PhraseService;

import com.google.common.base.Supplier;
import com.google.common.base.Suppliers;
import com.google.common.base.Throwables;

import static com.google.common.base.Charsets.UTF_8;
import static com.google.common.io.Files.asCharSource;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * File-backed {@link PhraseService}.
 *
 * @since 3.8
 */
@Component
@Qualifier("file")
public class FilePhraseService
    extends AbstractPhraseService
{
  private final Supplier<String> masterPhraseSupplier;

  @Autowired
  public FilePhraseService(@Value("${astrum.security.masterPhraseFile:#{null}}") @Nullable final File masterPhraseFile) {
    super(masterPhraseFile != null);
    this.masterPhraseSupplier = Suppliers.memoize(new Supplier<String>()
    {
      @Override
      public String get() {
        try {
          return asCharSource(masterPhraseFile, UTF_8).read().trim();
        }
        catch (IOException e) {
          throw Throwables.propagate(e);
        }
      }
    });
  }

  @Override
  protected String getMasterPhrase() {
    return masterPhraseSupplier.get();
  }
}
