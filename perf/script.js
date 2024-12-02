import http from "k6/http";
import { sleep, check, group } from "k6";
import { SharedArray } from "k6/data";

const API_URL = __ENV.API_URL;

const users = new SharedArray("users", function () {
  return JSON.parse(open("./users.json"));
});

const randomUser = () => users[Math.floor(Math.random() * users.length)];

export const options = {
  vus: 10000,
  duration: "1m",
};

export default function () {
  let userinfo = randomUser();

  group("1. Get user information (unauthorized)", () => {
    // Get user information. Expected to return 401 Unauthorized.
    let res = http.get(API_URL + "/user");

    check(res, {
      "user retrieve unauthorized": (r) => r.status == 401,
    });

    // 1-4 seconds for login.
    sleep(Math.random() * 3 + 1);
  });

  group("2. Login", () => {
    // Login to get API session token.
    let res = http.post(
      API_URL + "/login",
      JSON.stringify({
        username: userinfo.username,
        password: userinfo.password,
      }),
      {
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    check(res, {
      "login success status": (r) => r.status == 200,
      "set session cookie": (r) => r.cookies.session && r.cookies.session.length > 0,
    });

    check(res.json(), {
      "correct login response": (b) => b.data.username == userinfo.username,
    });

    // 0.2 seconds for redirect.
    sleep(0.2);
  });

  group("3. Get user information (authorized)", () => {
    // Get user information once again. Expected to succeed.
    let res = http.get(API_URL + "/user");

    check(res, {
      "user info retrieve successful": (r) => r.status == 200,
    });
    check(res.json(), {
      "correct user information response": (b) =>
        b.data.username == userinfo.username,
    });
  });

  group("4. Get tasks", () => {
    // Get a list of tasks.
    let res = http.get(API_URL + "/tasks");

    check(res, {
      "successful get tasks": (r) => r.status == 200,
    });
  });
}
