import cairosvg


def main():
    start = 5
    end = 30

    print("⏳ Converting {} to {}...".format(start, end))

    for i in range(start, end + 1):
        input_file = "child_{}.svg".format(i)
        output_file = "pattern_{}.png".format(i)

        cairosvg.svg2png(url=input_file, write_to=output_file)

        print("✅ Converted {} to {}".format(input_file, output_file))

    print("✅ Done!")


main()
