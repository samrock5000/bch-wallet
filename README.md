# Pre-alpha release ⚡️

Native Bitcoin Cash wallet 

defaults to localhost for electrum network access.
set the url in the config page ```my.electrumserver.com``` if connection is made a connect button will appear.
default address hd path:  m/44'/145'/0'/0/0 bchtest:....

# Integrate into Existing Project

If you already have an existing web project, this guide will walk you through integrating Tauri into your project, whether it is Node.js-based (like [Svelte], [React], [Vue], or [Angular]) or Rust-based (like [Yew] or [Dominator]).

:::info

Before we continue, make sure you have completed the [prerequisites] to have a working development environment.

:::

Although Tauri is compatible with nearly any frontend framework, we'll use a [React] project throughout this guide created using [create-react-app]. We'll be assuming you're starting with a project structure similar to this:

```
.
│── package.json
│── public
│   ╰── index.html
╰── src
    │── App.css
    │── App.jsx
    │── index.css
    ╰── index.js
```

## Create the Rust Project

<TauriInit
  destDirExplination={{
    __html:
      "For the project example in this guide, this is <code>../build</code>. Note that it may be something different like <code>../dist</code> if you're using a different framework.",
  }}
  devPathExplination={{
    __html:
      "For the project example in this guide, this is <code>http://localhost:3000</code>. Note that it may be something different (or even a directory) if you're using a different framework.",
  }}
  beforeDevCommandExplination={{
    __html:
      'For the project example in this guide, this is <code>npm run start</code> (make sure to adapt this to use the package manager of your choice).',
  }}
  beforeBuildCommandExplination={{
    __html:
      'For the project example in this guide, this is <code>npm run build</code> (make sure to adapt this to use the package manager of your choice).',
  }}
/>

And that's it, you have now added Tauri to your existing project and you should see a `src-tauri` directory that looks something like this:

```diff {9-14}
│── package.json
│── public
│   ╰── index.html
│── src
│   │── App.css
│   │── App.jsx
│   │── index.css
│   ╰── index.js
╰── src-tauri
    │── Cargo.toml
    │── build.rs
    │── icons
    │── src
    ╰── tauri.conf.json
```

## Invoke Commands

<Commands />

There are two different ways you can invoke commands from your frontend project:

1. Using the [`@tauri-apps/api`] JavaScript library (preferred)
2. Using [`withGlobalTauri`] to use a pre-bundled version of the Tauri API

We'll go through both below.

### Using JavaScript Library

To call our newly created command we will use the [`@tauri-apps/api`] JavaScript library. It provides access to core functionality such as windows, the filesystem, and more through convenient JavaScript abstractions. You can install it using your favorite JavaScript package manager:

<InstallTauriApi />

You can now import the `invoke` function from the library and use it to call our command:

```jsx title=src/App.jsx {4,7-12}
import logo from './logo.svg';
import './App.css';

import { invoke } from '@tauri-apps/api'

function App() {
  // now we can call our Command!
  // Right-click the application background and open the developer tools.
  // You will see "Hello, World!" printed in the console!
  invoke('greet', { name: 'World' })
  // `invoke` returns a Promise
  .then((response) => console.log(response))

  return (
    // -- snip --
  )
}
```

### Using `withGlobalTauri`

To interact with Tauri from your frontend without using the `@tauri-apps/api` JavaScript package you will need to enable [`withGlobalTauri`] in your `tauri.conf.json` file:

```json title=tauri.conf.json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:3000",
    "distDir": "../build",
    // highlight-next-line
    "withGlobalTauri": true
  },
```

This will inject a pre-bundled version of the API functions into your frontend.

You can now modify the `App.jsx` file to call your command:

```jsx title=src/App.js {4,7-12}
import logo from './logo.svg';
import './App.css';

// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri

function App() {
  // now we can call our Command!
  // Right-click the application background and open the developer tools.
  // You will see "Hello, World!" printed in the console!
  invoke('greet', { name: 'World' })
  // `invoke` returns a Promise
  .then((response) => console.log(response))

  return (
    // -- snip --
  )
}
```

## Running Your App

You can now run the following command in your terminal to start a development build of your app:

<Command name="dev" />

:::tip

If you want to know more about the communication between Rust and JavaScript, please read the Tauri [Inter-Process Communication][inter-process-communication] guide.

:::
