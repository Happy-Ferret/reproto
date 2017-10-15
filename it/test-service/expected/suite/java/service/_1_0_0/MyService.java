package service._1_0_0;

import common._1_0_0.Entry;
import java.util.concurrent.CompletableFuture;

public interface MyService {
  CompletableFuture<Void> foo(final Object request);

  CompletableFuture<Void> deleteId(final Entry request);
}
