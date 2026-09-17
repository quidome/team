/// <reference types="@sveltejs/kit" />

declare global {
  namespace App {
    interface Locals {
      coordinatorSession?: {
        subject: string;
      };
    }
  }
}

export {};
