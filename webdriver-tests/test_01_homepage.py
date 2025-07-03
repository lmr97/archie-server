import os
import time
import numpy as np
from selenium import webdriver as wd
from selenium.webdriver import ActionChains
from selenium.webdriver.remote.webelement import WebElement
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as ec


chrome_opts  = wd.ChromeOptions()
firefox_opts = wd.FirefoxOptions()
edge_opts    = wd.EdgeOptions()

drivers = [
    wd.Remote(command_executor="http://127.0.0.1:4444", options=chrome_opts),
    #wd.Remote(command_executor="http://127.0.0.1:4445", options=firefox_opts),
    #wd.Remote(command_executor="http://127.0.0.1:4446", options=edge_opts)
    ]


root_url = os.getenv("DOCKER_SERVER_URL")   # has no trailing slash
print(root_url)

for d in drivers:
    d.get(root_url)
    time.sleep(1)    # make sure the hit can be posted to the DB after each visit


def test_hit_count():

    correct_hits = 7
    for drv in drivers:

        hit_count = drv.find_element(By.ID, "hit-count")

        # started with 6 hits in the DB, and both browsers navigated 
        # to the page during init
        assert str(correct_hits) in hit_count.text, f"failed for {drv.name}"
        
        correct_hits += 1 # increment, since each browser saw different count


# test whether the logo enlarges when moused over,
# and shows the message from out behind it
def test_react_logo():
    for drv in drivers:
        react_logo_div: WebElement = drv.find_element(By.CSS_SELECTOR, "div.interact-synchronizer")
        react_logo: WebElement     = react_logo_div.find_element(By.CSS_SELECTOR, "svg.react-logo")
        viewbox_before: str        = react_logo.get_dom_attribute("viewBox")
        viewbox_dims_before        = np.array(viewbox_before.split(" "), dtype=float)  # using Numpy for comparisons

        # hover over
        ActionChains(drv) \
            .move_to_element(react_logo_div) \
            .perform()

        # give it a moment to change in up to 2 browsers (it's gradual)
        logo_msg = react_logo_div.find_element(By.CSS_SELECTOR, "p.react-msg")
        wait     = WebDriverWait(drv, 5.0)
        wait.until(ec.visibility_of(logo_msg))

        viewbox_after: str = react_logo.get_dom_attribute("viewBox")
        viewbox_dims_after = np.array(viewbox_after.split(" "), dtype=float)

        # did the logo enlarge? 
        assert (viewbox_dims_before[:2] < viewbox_dims_after[:2]).all(), f"failed for {drv.name}"
        assert (viewbox_dims_before[2:] > viewbox_dims_after[2:]).all(), f"failed for {drv.name}"

        # did the message appear?
        assert react_logo_div \
            .find_element(By.CSS_SELECTOR, "p.react-msg") \
            .is_displayed(), \
            f"failed for {drv.name}"


def test_teardown():
    for drv in drivers:
        drv.quit()
