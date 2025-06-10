import '@testing-library/jest-dom/vitest'
import * as matchers from '@testing-library/jest-dom/matchers'
import { describe, expect, it } from "vitest";

expect.extend(matchers);

describe("Init", () => it("gets vitest to not throw a fit"))