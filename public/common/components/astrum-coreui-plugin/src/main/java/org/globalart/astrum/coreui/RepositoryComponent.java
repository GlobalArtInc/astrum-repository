
package org.globalart.astrum.coreui;

import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.globalart.astrum.coreui.search.BrowseableFormatXO;
import org.globalart.astrum.coreui.service.RepositoryUiService;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.extdirect.model.StoreLoadParameters;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import com.softwarementors.extjs.djn.config.annotations.DirectPollMethod;
import org.apache.shiro.authz.annotation.RequiresAuthentication;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Repository {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_Repository")
public class RepositoryComponent
    extends DirectComponentSupport
{
  private final RepositoryUiService repositoryUiService;

  @Autowired
  public RepositoryComponent(final RepositoryUiService repositoryUiService) {
    this.repositoryUiService = checkNotNull(repositoryUiService);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<RepositoryXO> read() {
    return repositoryUiService.read();
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<ReferenceXO> readRecipes() {
    return repositoryUiService.readRecipes();
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<Format> readFormats() {
    return repositoryUiService.readFormats();
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<BrowseableFormatXO> getBrowseableFormats() {
    return repositoryUiService.getBrowseableFormats();
  }

  /**
   * Retrieve a list of available repositories references.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<RepositoryReferenceXO> readReferences(@Nullable final StoreLoadParameters parameters) {
    return repositoryUiService.readReferences(parameters);
  }

  /**
   * Retrieve a list of available repositories references + add an entry for all repositories '*".
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<RepositoryReferenceXO> readReferencesAddingEntryForAll(@Nullable final StoreLoadParameters parameters) {
    return repositoryUiService.readReferencesAddingEntryForAll(parameters);
  }

  /**
   * Retrieve a list of available repositories references + add an entry for all repositories '*' and an entry for
   * format 'All (format) repositories' '*(format)'".
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<RepositoryReferenceXO> readReferencesAddingEntriesForAllFormats(
      @Nullable final StoreLoadParameters parameters)
  {
    return repositoryUiService.readReferencesAddingEntriesForAllFormats(parameters);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @Validate(groups = {Create.class, Default.class})
  public RepositoryXO create(@NotNull @Valid final RepositoryXO repositoryXO) throws Exception {
    return repositoryUiService.create(repositoryXO);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @Validate(groups = {Update.class, Default.class})
  public RepositoryXO update(@NotNull @Valid final RepositoryXO repositoryXO) throws Exception {
    return repositoryUiService.update(repositoryXO);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @Validate
  public void remove(@NotEmpty final String name) throws Exception {
    repositoryUiService.remove(name);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @Validate
  public String rebuildIndex(@NotEmpty final String name) {
    return repositoryUiService.rebuildIndex(name);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @Validate
  public void invalidateCache(@NotEmpty final String name) {
    repositoryUiService.invalidateCache(name);
  }

  @Timed
  @ExceptionMetered
  @DirectPollMethod(event = "coreui_Repository_readStatus")
  @RequiresAuthentication
  public List<RepositoryStatusXO> readStatus(final Map<String, String> params) {
    return repositoryUiService.readStatus(params);
  }

  public void addRecipe(String format, Recipe recipe) {
    repositoryUiService.addRecipe(format, recipe);
  }

  public RepositoryUiService getRepositoryUiService() {
    return repositoryUiService;
  }
}
