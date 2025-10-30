import os
from time import sleep
from selenium import webdriver as wd
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as ec


chrome_opts  = wd.ChromeOptions()
firefox_opts = wd.FirefoxOptions()
edge_opts    = wd.EdgeOptions()

# necessary to allow this script to download the CSV files
chrome_opts.enable_downloads = True
firefox_opts.enable_downloads = True
edge_opts.enable_downloads = True


drivers = [
    wd.Remote(command_executor="http://127.0.0.1:4444", options=chrome_opts),
    wd.Remote(command_executor="http://127.0.0.1:4445", options=firefox_opts),
    wd.Remote(command_executor="http://127.0.0.1:4446", options=edge_opts)
    ]

root_url = os.getenv("DOCKER_SERVER_URL")   # has no trailing slash
[d.get(root_url+"/lb-list-conv") for d in drivers]

ALL_ATTRS_LIST_EXISTS = False

def unordered_row_val_compare(true_row: str, test_row: str, attrs: list[str], is_ranked: bool):
    
    headers = ["title", "year"] + attrs
    if is_ranked:
        headers = ["rank"] + headers

    true_row_fields = true_row.split(",")
    test_row_fields = test_row.split(",")
    
    for (field, true_value, test_value) in zip(headers, true_row_fields, test_row_fields):
        true_value    = true_value.replace("\"", "")  # shave off double-quotes
        test_value    = test_value.replace("\"", "")
        true_value    = true_value.replace("\n", "")  # take out newlines
        test_value    = test_value.replace("\n", "")
        true_val_list = true_value.split("; ")
        test_val_list = test_value.split("; ")
        assert set(true_val_list) == set(test_val_list), f"failed for field: {field}"


def test_normal_list_no_attrs():

    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.send_keys("https://letterboxd.com/dialectica972/list/test-list-all-attributes/")
        
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait until file is downloaded (submit button reappears)
        wait = WebDriverWait(drv, 15.0)
        wait.until(
            ec.presence_of_element_located(
                (By.CSS_SELECTOR, "button[type='submit']")
            )
        )

        sleep(1)   # to try an address what seems to be a race condition
        drv.download_file("test-list-all-attributes.csv","./downloads")
        
        # determine filename in later tests
        global ALL_ATTRS_LIST_EXISTS
        ALL_ATTRS_LIST_EXISTS = True

        true_data = []
        test_data = []
        
        with open("../test-helpers/short-list-no-attrs.csv", "r", encoding="utf-8") as true_file:
            true_data = true_file.readlines()

        with open("./downloads/test-list-all-attributes.csv") as test_file:
            test_data = test_file.readlines()

        for (i, true_row) in enumerate(true_data):
            assert true_row == test_data[i], f"Test failed for {drv.name}"


def test_normal_list_some_attrs():
    
    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://letterboxd.com/dialectica972/list/test-list-all-attributes/")
        
        # click some attribute boxes
        # should be alphabetized upon download
        attrs = ["genre", "director", "avg-rating", "writer"]
        for attr in attrs:
            drv.find_element(By.NAME, attr).click()
        
        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait until file is downloaded (submit button reappears)
        wait = WebDriverWait(drv, 15.0)
        wait.until(
            ec.presence_of_element_located(
                (By.CSS_SELECTOR, "button[type='submit']")
            )
        )

        sleep(1)   # to try an address what seems to be a race condition

        # allows this test to be run independently
        if not ALL_ATTRS_LIST_EXISTS:
            drv.download_file("test-list-all-attributes.csv","./downloads")
        else:
            # on Firefox, there is no space between the title and the (1)
            if drv.name == "firefox":
                drv.download_file("test-list-all-attributes(1).csv","./downloads")
            else:
                drv.download_file("test-list-all-attributes (1).csv","./downloads")

        true_data = []
        test_data = []
        
        with open("./short-list-some-attrs.csv", "r", encoding="utf-8") as true_file:
            true_data = true_file.readlines()

        with open("./downloads/test-list-all-attributes (1).csv") as test_file:
            test_data = test_file.readlines()


        for (true_row, test_row) in zip(true_data, test_data):
            try:
                assert true_row == test_row, f"Test failed for {drv.name}"
            
            # give it a closer look on failure, to see where in the row
            # is not matching the testing constants
            except AssertionError:
                
                unordered_row_val_compare(true_row, test_row, attrs, is_ranked=False)


# some attributes included
def test_ranked_list():
    
    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://letterboxd.com/dialectica972/list/testing-a-ranked-list/")

        # check some different boxes this time
        attrs = ["theme", "songs", "editor", "country"]
        for attr in attrs:
            drv.find_element(By.NAME, attr).click()


        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait until file is downloaded (submit button reappears)
        wait = WebDriverWait(drv, 15.0)
        wait.until(
            ec.presence_of_element_located(
                (By.CSS_SELECTOR, "button[type='submit']")
            )
        )

        sleep(1)   # to try an address what seems to be a race condition
        drv.download_file("testing-a-ranked-list.csv","./downloads")

        true_data = []
        test_data = []
        
        with open("./ranked-list.csv", "r", encoding="utf-8") as true_file:
            true_data = true_file.readlines()

        with open("./downloads/testing-a-ranked-list.csv") as test_file:
            test_data = test_file.readlines()


        for (true_row, test_row) in zip(true_data, test_data):
            try:
                assert true_row == test_row, f"Test failed for {drv.name}"
            
            # give it a closer look on failure, to see where in the row
            # is not matching the testing constants, if it's not a matter of ordering
            except AssertionError:
                
                unordered_row_val_compare(true_row, test_row, attrs, is_ranked=True)


def test_overlong_list():

    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://letterboxd.com/maxwren/list/monster-mega-list-2-actual-watch-list-1/")

        # check some other different boxes this time
        drv.find_element(By.NAME, "casting").click()
        drv.find_element(By.NAME, "art-direction").click()

        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait for alert to show up
        wait  = WebDriverWait(drv, timeout=3.0)
        alert = wait.until(ec.alert_is_present())

        # as long as the basics are covered
        assert ("not accept" in alert.text) and ("10,000 films" in alert.text)

        # close alert box, and return focus to the page as a whole
        alert.accept()


def test_invalid_list():
    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://letterboxd.com/invaliduser/list/list-that-doesnt-exist/")

        # check some other different boxes this time
        drv.find_element(By.NAME, "producer").click()
        drv.find_element(By.NAME, "assistant-director").click()

        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait for alert to show up
        wait  = WebDriverWait(drv, timeout=3.0)
        alert = wait.until(ec.alert_is_present())

        # as long as the basics are covered
        assert ("doesn't appear to be a valid Letterboxd list" in alert.text)

        # close alert box, and return focus to the page as a whole
        alert.accept()


def test_non_letterboxd_url():
    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://example.com/not/a/list/on/letterboxd/")

        # check some other different boxes this time
        drv.find_element(By.NAME, "makeup").click()
        drv.find_element(By.NAME, "language").click()

        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait for HTML validation error to show up (not a pop-up)
        # adapted from the Java version from this SO answer:
        # https://stackoverflow.com/a/62858110/20496903
        wait = WebDriverWait(drv, timeout=2.0)
        wait.until(ec.element_to_be_clickable(url_box))

        val_error_main  = url_box.get_attribute("validationMessage")
        val_error_extra = url_box.get_dom_attribute("title")  # the custom text added to the error
        print(val_error_main)
        
        assert ("Please match the requested format" in val_error_main)
        assert ("A valid URL for a list on Letterboxd.com" in val_error_extra)



def test_teardown():

    for drv in drivers:
        drv.quit()