package test;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import java.util.Objects;
import java.util.Optional;

@JsonTypeInfo(use=JsonTypeInfo.Id.NAME, include=JsonTypeInfo.As.PROPERTY, property="type")
@JsonSubTypes({@JsonSubTypes.Type(name="one", value=Interface.One.class), @JsonSubTypes.Type(name="two", value=Interface.Two.class)})
public interface Interface {
  public String getName();

  public static class One implements Interface {
    private final String name;
    private final Data data;

    @JsonCreator
    public One(
      @JsonProperty("name") final String name, 
      @JsonProperty("data") final Data data
    ) {
      Objects.requireNonNull(name, "name");
      this.name = name;
      Objects.requireNonNull(data, "data");
      this.data = data;
    }

    @Override
    public String getName() {
      return this.name;
    }

    public String getName() {
      return this.name;
    }

    public Data getData() {
      return this.data;
    }

    @Override
    public int hashCode() {
      int result = 1;
      result = result * 31 + this.name.hashCode();
      result = result * 31 + this.data.hashCode();
      return result;
    }

    @Override
    public boolean equals(final Object other) {
      if (other == null) {
        return false;
      }

      if (!(other instanceof Interface.One)) {
        return false;
      }

      @SuppressWarnings("unchecked")
      final Interface.One o = (Interface.One) other;

      if (!this.name.equals(o.name)) {
        return false;
      }

      if (!this.data.equals(o.data)) {
        return false;
      }

      return true;
    }

    @Override
    public String toString() {
      final StringBuilder b = new StringBuilder();

      b.append("Interface.One");
      b.append("(");
      b.append("name=");
      b.append(this.name.toString());
      b.append(", ");
      b.append("data=");
      b.append(this.data.toString());
      b.append(")");

      return b.toString();
    }

    public static class Builder {
      private Optional<String> name = Optional.empty();
      private Optional<Data> data = Optional.empty();

      public Builder name(final String name) {
        this.name = Optional.of(name);
        return this;
      }

      public Builder data(final Data data) {
        this.data = Optional.of(data);
        return this;
      }

      public Interface.One build() {
        final String name = this.name.orElseThrow(() -> new RuntimeException("name: is required"));
        final Data data = this.data.orElseThrow(() -> new RuntimeException("data: is required"));

        return new Interface.One(name, data);
      }
    }
  }

  public static class Two implements Interface {
    private final String name;
    private final Data data;

    @JsonCreator
    public Two(
      @JsonProperty("name") final String name, 
      @JsonProperty("data") final Data data
    ) {
      Objects.requireNonNull(name, "name");
      this.name = name;
      Objects.requireNonNull(data, "data");
      this.data = data;
    }

    @Override
    public String getName() {
      return this.name;
    }

    public String getName() {
      return this.name;
    }

    public Data getData() {
      return this.data;
    }

    @Override
    public int hashCode() {
      int result = 1;
      result = result * 31 + this.name.hashCode();
      result = result * 31 + this.data.hashCode();
      return result;
    }

    @Override
    public boolean equals(final Object other) {
      if (other == null) {
        return false;
      }

      if (!(other instanceof Interface.Two)) {
        return false;
      }

      @SuppressWarnings("unchecked")
      final Interface.Two o = (Interface.Two) other;

      if (!this.name.equals(o.name)) {
        return false;
      }

      if (!this.data.equals(o.data)) {
        return false;
      }

      return true;
    }

    @Override
    public String toString() {
      final StringBuilder b = new StringBuilder();

      b.append("Interface.Two");
      b.append("(");
      b.append("name=");
      b.append(this.name.toString());
      b.append(", ");
      b.append("data=");
      b.append(this.data.toString());
      b.append(")");

      return b.toString();
    }

    public static class Builder {
      private Optional<String> name = Optional.empty();
      private Optional<Data> data = Optional.empty();

      public Builder name(final String name) {
        this.name = Optional.of(name);
        return this;
      }

      public Builder data(final Data data) {
        this.data = Optional.of(data);
        return this;
      }

      public Interface.Two build() {
        final String name = this.name.orElseThrow(() -> new RuntimeException("name: is required"));
        final Data data = this.data.orElseThrow(() -> new RuntimeException("data: is required"));

        return new Interface.Two(name, data);
      }
    }
  }
}
