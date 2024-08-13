for scene in dragon cornell-box
do
    for spp in 5 20 100
    do 
        echo $scene $spp
        cargo run --release -- --scene $scene --output file --spp $spp -d 1920x1080 
        output=output/$(echo $scene | sed 's/-.\+//')$spp
        mkdir -p $output
        mv output/ldr/* $output
    done
done 
