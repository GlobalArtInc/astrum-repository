
package org.globalart.astrum.repository.tools;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.3
 */
public class DeadBlobResult<T>
{
  private final String repositoryName;

  private final T asset;

  private final String errorMessage;

  private final ResultState resultState;

  public DeadBlobResult(
      final String repositoryName,
      @Nullable final T asset,
      final ResultState resultState,
      final String errorMessage)
  {
    this.repositoryName = checkNotNull(repositoryName);
    this.asset = asset;
    this.resultState = checkNotNull(resultState);
    this.errorMessage = errorMessage;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public T getAsset() {
    return asset;
  }

  public String getErrorMessage() {
    return errorMessage;
  }

  public ResultState getResultState() {
    return resultState;
  }

  @Override
  public String toString() {
    return "DeadBlobResult [repositoryName=" + repositoryName + ", asset=" + asset + ", errorMessage=" + errorMessage
        + ", resultState=" + resultState + "]";
  }
}
