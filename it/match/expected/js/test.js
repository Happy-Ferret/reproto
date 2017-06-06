class Data {
  constructor(name) {
    this.name = name;
  }

  static decode(data) {
    const name = data["name"];

    return new Data(name);
  }

  encode() {
    const data = {};

    if (this.name === null || this.name === undefined) {
      throw new Error("name: is a required field");
    }

    data["name"] = this.name;

    return data;
  }
}

class Point {
  constructor(timestamp, value) {
    this.timestamp = timestamp;
    this.value = value;
  }

  static decode(data) {
    if (data == 42) {
      return new Point(42, 41.2)
    }

    if (typeof data === "number") {
      n = data
      return new Point(n, 42);
    }

    if (data.constructor === Object) {
      p = Point.decode(data)
      return p;
    }

    const timestamp = data[0];

    const value = data[1];

    return new Point(timestamp, value);
  }

  encode() {
    if (this.timestamp === null || this.timestamp === undefined) {
      throw new Error("TS: is a required field");
    }

    if (this.value === null || this.value === undefined) {
      throw new Error("value: is a required field");
    }

    return [this.timestamp, this.value];
  }
}

class Interface {
  static decode(data) {
    const f_type = data["type"]

    if (f_type === "one") {
      return One.decode(data);
    }

    if (f_type === "two") {
      return Two.decode(data);
    }

    throw new Error("bad type");
  }
}

class One {
  constructor(name, data) {
    this.name = name;
    this.data = data;
  }

  static decode(data) {
    const name = data["name"];

    const data = Data.decode(data["data"]);

    return new One(name, data);
  }

  encode() {
    const data = {};

    data["type"] = One.TYPE;

    if (this.name === null || this.name === undefined) {
      throw new Error("name: is a required field");
    }

    data["name"] = this.name;

    if (this.data === null || this.data === undefined) {
      throw new Error("data: is a required field");
    }

    data["data"] = this.data.encode();

    return data;
  }
}

One.TYPE = "One";

class Two {
  constructor(name, data) {
    this.name = name;
    this.data = data;
  }

  static decode(data) {
    const name = data["name"];

    const data = Data.decode(data["data"]);

    return new Two(name, data);
  }

  encode() {
    const data = {};

    data["type"] = Two.TYPE;

    if (this.name === null || this.name === undefined) {
      throw new Error("name: is a required field");
    }

    data["name"] = this.name;

    if (this.data === null || this.data === undefined) {
      throw new Error("data: is a required field");
    }

    data["data"] = this.data.encode();

    return data;
  }
}

Two.TYPE = "Two";

class Type {
  constructor(data) {
    this.data = data;
  }

  static decode(data) {
    if (typeof data === "string") {
      data = data
      return new Type(data);
    }

    const data = data["data"];

    return new Type(data);
  }

  encode() {
    const data = {};

    if (this.data === null || this.data === undefined) {
      throw new Error("data: is a required field");
    }

    data["data"] = this.data;

    return data;
  }
}
